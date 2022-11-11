use std::{collections::HashMap, hash::Hash, io::Read};

use macroquad::{prelude::*, rand::gen_range};
use serde::Deserialize;

use crate::{
    graphics::{draw_body, draw_body_texture, draw_circ, draw_lin, draw_rect, Screen},
    util::{
        Ball, Enemy, Form, MoveAction, Player, PlayerAction, BALL_RADIUS, BALL_SPEED,
        PLAYER_RADIUS, PLAYER_RELOAD, RATIO_W_H, SLASH_LEN, WALL_SIZE,
    },
};

#[derive(Debug)]
pub enum Error {
    Parse(serde_yaml::Error),
    NoEntry,
    MoreThanOneEntry(Vec<u8>),
    TwoDoorsWithSameDirection {
        direction: Direction,
        doors: (u8, u8),
    },
}

pub struct Level {
    config: LevelConfig,
    rooms: Vec<Room>,
    current_room: usize,
    player: Player,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

pub struct Room {
    id: u8,
    enemies: Vec<Enemy>,
    balls: Vec<Ball>,
    doors: HashMap<Direction, usize>,
}

#[derive(Deserialize, Clone)]
struct LevelConfig {
    rooms: Vec<RoomConfig>,
}

#[derive(Clone, Deserialize)]
struct RoomConfig {
    id: u8,
    enter: Option<Direction>,
    doors: Vec<DoorConfig>,
    enemies: u8,
}

impl PartialEq for RoomConfig {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for RoomConfig {}

impl Hash for RoomConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Clone, Deserialize)]
struct DoorConfig {
    direction: Direction,
    to: u8,
}

impl Level {
    pub async fn from_string(str: impl AsRef<str>) -> Result<Self, Error> {
        let config = serde_yaml::from_str(str.as_ref()).map_err(Error::Parse)?;
        Self::from_config(config).await
    }

    pub async fn from_reader<R: Read>(reader: R) -> Result<Self, Error> {
        let config = serde_yaml::from_reader(reader).map_err(Error::Parse)?;
        Self::from_config(config).await
    }

    async fn from_config(config: LevelConfig) -> Result<Self, Error> {
        let rooms = &config.rooms;
        let room_map = rooms
            .iter()
            .map(|room| {
                (
                    room,
                    rooms
                        .iter()
                        .filter_map(|connected| {
                            room.doors
                                .iter()
                                .find(|door| door.to == connected.id)
                                .map(|door| (door.direction, connected))
                                .or_else(|| {
                                    connected
                                        .doors
                                        .iter()
                                        .find(|door| door.to == room.id)
                                        .map(|door| (door.direction.inverse(), connected))
                                })
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mut enters: Vec<_> = rooms.iter().filter(|room| room.enter.is_some()).collect();
        let entry_room = match enters.len() {
            1 => enters.pop().unwrap(),
            0 => return Err(Error::NoEntry),
            _ => {
                return Err(Error::MoreThanOneEntry(
                    enters.iter().map(|room| room.id).collect(),
                ))
            }
        };
        let Some(enter) = entry_room.enter else {
            unreachable!()
        };
        let randomed = gen_range(0.35, 0.65);
        let position = match enter {
            Direction::North => Vec2 {
                x: randomed,
                y: 0.1,
            },
            Direction::South => Vec2 {
                x: randomed,
                y: 0.9,
            },
            Direction::West => Vec2 {
                x: 0.1,
                y: randomed,
            },
            Direction::East => Vec2 {
                x: 0.9,
                y: randomed,
            },
        };
        let player = Player::new(position).await;
        let mut result_rooms = Vec::with_capacity(rooms.len());
        let current_room = push_room(&mut result_rooms, entry_room, &room_map)?;
        Ok(Self {
            config,
            rooms: result_rooms,
            current_room,
            player,
        })
    }
    pub async fn restart(&mut self) {
        *self = Level::from_config(self.config.clone()).await.unwrap();
    }

    pub fn update(&mut self, player_action: PlayerAction, dt: f32) -> Option<bool> {
        if self
            .rooms
            .iter()
            .all(|room| room.enemies.iter().all(|enemy| enemy.dead))
        {
            return Some(true);
        }

        let x_direction = player_action.view_point.x - self.player.body.position.x;
        let y_direction = player_action.view_point.y - self.player.body.position.y;

        let move_action = MoveAction {
            move_direction: player_action.move_direction,
            sight: Vec2 {
                x: x_direction,
                y: y_direction,
            }
            .normalize_or_zero(),
        };
        self.player.body.update(move_action, dt);
        let door = self.rooms[self.current_room]
            .doors
            .iter()
            .find(|(direction, _)| {
                let (x_range, y_range) = match direction {
                    Direction::North => (
                        (RATIO_W_H / 2. - 0.15..=RATIO_W_H / 2. + 0.15),
                        (0.0..=WALL_SIZE + 0.05),
                    ),
                    Direction::South => (
                        (RATIO_W_H / 2. - 0.15..=RATIO_W_H / 2. + 0.15),
                        ((1.0 - WALL_SIZE - 0.05)..=1.0),
                    ),
                    Direction::East => {
                        (((RATIO_W_H - WALL_SIZE - 0.05)..=RATIO_W_H), (0.35..=0.65))
                    }
                    Direction::West => ((0.0..=(WALL_SIZE + 0.05)), (0.35..=0.65)),
                };
                x_range.contains(&self.player.body.position.x)
                    && y_range.contains(&self.player.body.position.y)
            });
        if let Some((direction, to)) = door {
            match direction {
                Direction::North | Direction::South => {
                    self.player.body.position.y = clamp(1. - self.player.body.position.y, 0.1, 0.9);
                }
                Direction::East | Direction::West => {
                    self.player.body.position.x = clamp(
                        RATIO_W_H - self.player.body.position.x,
                        0.1,
                        RATIO_W_H - 0.1,
                    );
                }
            }
            self.current_room = *to;
        }

        self.rooms.iter_mut().for_each(|room| {
            room.balls = room
                .balls
                .iter()
                .filter(|ball| {
                    ball.position.x >= WALL_SIZE + PLAYER_RADIUS
                        && ball.position.x <= RATIO_W_H - WALL_SIZE - PLAYER_RADIUS
                        && ball.position.y >= WALL_SIZE + PLAYER_RADIUS
                        && ball.position.y <= 1. - WALL_SIZE - PLAYER_RADIUS
                })
                .map(|ball| Ball {
                    position: ball.position + ball.direction * BALL_SPEED * dt,
                    direction: ball.direction,
                })
                .collect();
        });

        for (room_id, room) in self.rooms.iter_mut().enumerate() {
            for enemy in &mut room.enemies {
                let (action, slash) = if enemy.dead {
                    (
                        MoveAction {
                            move_direction: (0, 0),
                            sight: enemy.body.sight,
                        },
                        false,
                    )
                } else {
                    let player = (room_id == self.current_room
                        && (self.player.visible
                            || enemy.body.position.distance(self.player.body.position)
                                < 2. * PLAYER_RADIUS + SLASH_LEN / 2.))
                        .then_some(&self.player.body);
                    enemy.actor.action(&enemy.body, player, dt)
                };
                enemy.body.update(action, dt);
                if room_id == self.current_room {
                    enemy.body.collide(&mut self.player.body);
                }
                if slash && enemy.reload == 0. {
                    enemy.reload = PLAYER_RELOAD;
                    enemy.slash = 5;
                    if self.player.low_health {
                        return Some(false);
                    }
                    self.player.low_health = true;
                } else {
                    enemy.slash = clamp(enemy.slash - 1, 0, 5);
                    enemy.reload = clamp(enemy.reload - dt, 0., PLAYER_RELOAD);
                }

                let mut ball_collisions = Vec::new();
                for ball in &room.balls {
                    if enemy.body.position.distance(ball.position) < BALL_RADIUS + PLAYER_RADIUS {
                        enemy.dead = true;
                        ball_collisions.push(ball.position);
                    }
                }
                room.balls
                    .retain(|ball| !ball_collisions.contains(&ball.position));
            }
            let mut enemy_shifts: Vec<_> =
                (0..room.enemies.len()).map(|_| Vec2::default()).collect();
            for (left_pos, left) in room.enemies.iter().enumerate() {
                for (right_pos, right) in room.enemies.iter().enumerate() {
                    if left == right {
                        continue;
                    }
                    if let Some(shift) = left.body.collision(&right.body) {
                        enemy_shifts[left_pos] += shift;
                        enemy_shifts[right_pos] -= shift;
                    }
                }
            }
            for (enemy_pos, enemy) in room.enemies.iter_mut().enumerate() {
                enemy.body.position += enemy_shifts[enemy_pos] / 2.;
            }
        }
        if player_action.toggle_visibility {
            self.player.visible = !self.player.visible;
            self.player.body.form = if self.player.visible {
                Form::Rect {
                    width: PLAYER_RADIUS,
                    height: 1.5 * PLAYER_RADIUS,
                }
            } else {
                Form::Rect {
                    width: 1.5 * PLAYER_RADIUS,
                    height: 1.5 * PLAYER_RADIUS,
                }
            };
        }
        if player_action.shoot && self.player.visible && self.player.reload == 0. {
            self.player.reload = PLAYER_RELOAD;
            let position = self.player.body.position + (self.player.body.sight * PLAYER_RADIUS);
            self.rooms[self.current_room].balls.push(Ball {
                position,
                direction: self.player.body.sight,
            });
        } else {
            self.player.reload = clamp(self.player.reload - dt, 0., PLAYER_RELOAD);
        }

        None
    }

    pub fn draw(&self, screen: &Screen) {
        draw_rect(
            screen,
            0.,
            0.,
            RATIO_W_H,
            1.,
            if self.player.low_health { RED } else { GRAY },
        );
        draw_rect(
            screen,
            WALL_SIZE,
            WALL_SIZE,
            RATIO_W_H - 2. * WALL_SIZE,
            1. - 2. * WALL_SIZE,
            WHITE,
        );

        // draw_body(
        //     screen,
        //     &self.player.body,
        //     if self.player.visible { GREEN } else { BLUE },
        // );
        draw_body_texture(
            screen,
            &self.player.body,
            self.player.model,
            WHITE,
            if self.player.visible {
                Rect {
                    x: 10.0,
                    y: 10.0,
                    w: 100.0,
                    h: 150.0,
                }
            } else {
                Rect {
                    x: 120.0,
                    y: 10.0,
                    w: 150.0,
                    h: 150.0,
                }
            },
        );

        // INFO: uncomment if want to see sight trace
        //
        // let (x_mouse, y_mouse) = {
        //     let (x_m, y_m) = mouse_position();
        //     (
        //         clamp((x_m - screen.x) / screen.height, 0., RATIO_W_H),
        //         clamp((y_m - screen.y) / screen.height, 0., 1.),
        //     )
        // };
        // draw_lin(
        //     screen,
        //     state.player.position.x,
        //     state.player.position.y,
        //     x_mouse,
        //     y_mouse,
        //     BALL_RADIUS,
        //     GRAY,
        // );
        let room = &self.rooms[self.current_room];
        for ball in &room.balls {
            draw_circ(screen, ball.position.x, ball.position.y, BALL_RADIUS, RED);
        }
        for enemy in &room.enemies {
            draw_body(screen, &enemy.body, ORANGE);
            if enemy.dead {
                draw_circ(
                    screen,
                    enemy.body.position.x,
                    enemy.body.position.y,
                    PLAYER_RADIUS / 2.,
                    RED,
                );
            }
            if enemy.slash > 0 {
                let slash_x = enemy.body.sight.x * PLAYER_RADIUS + enemy.body.position.x;
                let slash_y = enemy.body.sight.y * PLAYER_RADIUS + enemy.body.position.y;
                let slash_x_end = enemy.body.sight.x * SLASH_LEN + slash_x;
                let slash_y_end = enemy.body.sight.y * SLASH_LEN + slash_y;
                draw_lin(
                    screen,
                    slash_x,
                    slash_y,
                    slash_x_end,
                    slash_y_end,
                    0.05,
                    GRAY,
                );
            }
        }
        for direction in room.doors.keys() {
            let (x, y, w, h) = match direction {
                Direction::North => (RATIO_W_H / 2. - 0.15, 0.0, 0.3, WALL_SIZE),
                Direction::South => (RATIO_W_H / 2. - 0.15, 1.0 - WALL_SIZE, 0.3, WALL_SIZE),
                Direction::East => (RATIO_W_H - WALL_SIZE, 0.5 - 0.15, WALL_SIZE, 0.3),
                Direction::West => (0.0, 0.5 - 0.15, WALL_SIZE, 0.3),
            };
            draw_rect(screen, x, y, w, h, BROWN);
        }
    }
}

fn push_room(
    rooms: &mut Vec<Room>,
    room: &RoomConfig,
    room_map: &HashMap<&RoomConfig, Vec<(Direction, &RoomConfig)>>,
) -> Result<usize, Error> {
    let mut connected_rooms = HashMap::new();
    for (direction, room) in room_map.get(room).unwrap().iter().cloned() {
        if let Some(previous) = connected_rooms.insert(direction, room) {
            return Err(Error::TwoDoorsWithSameDirection {
                direction,
                doors: (previous.id, room.id),
            });
        }
    }
    rooms.push(Room {
        id: room.id,
        enemies: (0..room.enemies)
            .map(|id| {
                Enemy::new(
                    30 * rooms.len() as u32 + id as u32,
                    Vec2 {
                        x: gen_range(RATIO_W_H / 3.0, 2. * RATIO_W_H / 3.),
                        y: gen_range(0.25, 0.75),
                    },
                )
            })
            .collect(),
        balls: Vec::new(),
        doors: HashMap::new(),
    });
    let room_pos = rooms.len() - 1;
    rooms[room_pos].doors = connected_rooms
        .into_iter()
        .map(|(direction, room)| {
            let room = rooms
                .iter()
                .position(|r| r.id == room.id)
                .map_or_else(|| push_room(rooms, room, room_map), Ok)?;
            Ok((direction, room))
        })
        .collect::<Result<_, _>>()?;
    Ok(room_pos)
}
impl Direction {
    const fn inverse(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}
