use std::{collections::HashMap, hash::Hash};

use bevy_ecs::{
    query::With,
    system::{Query, Res},
};
use macroquad::{prelude::*, rand::gen_range};
use serde::Deserialize;

use crate::{
    assets::Assets,
    graphics::{draw_circ, draw_rect, draw_txt, get_lines, Screen},
    util::{
        Ball2, Body2, Direction, Door, Enemy2, EnemyBundle, EnemyState, Form, Health, Phrase,
        Player2, Position, Post, Reload, Room, Sight, Speed, Visible, BALL_RADIUS, PLAYER_RADIUS,
        RATIO_W_H, WALL_SIZE,
    },
};

#[derive(Deserialize, Clone)]
pub struct LevelConfig {
    pub rooms: Vec<RoomConfig>,
}

#[derive(Clone, Deserialize)]
pub struct RoomConfig {
    pub id: u8,
    pub enter: Option<Direction>,
    pub doors: Vec<DoorConfig>,
    pub enemies: u8,
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
pub struct DoorConfig {
    pub direction: Direction,
    pub to: u8,
}

pub fn draw_player(
    player: Query<(&Position, &Form, &Sight, Option<&Visible>), With<Player2>>,
    screen: Res<Screen>,
    assets: Res<Assets>,
) {
    let Ok((position, form, sight, visible )) =
        player.get_single() else {
        return;
    };
    let visible = visible.is_some();
    draw_texture_ex(
        assets.images["player"],
        (position.0.x - form.x_r()) * screen.height + screen.x,
        (position.0.y - form.y_r()) * screen.height + screen.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2 {
                x: 2. * form.x_r() * screen.height,
                y: 2. * form.y_r() * screen.height,
            }),
            source: if visible {
                Some(Rect {
                    x: 10.,
                    y: 10.,
                    w: 100.,
                    h: 150.,
                })
            } else {
                Some(Rect {
                    x: 120.,
                    y: 10.,
                    w: 150.,
                    h: 150.,
                })
            },
            flip_x: sight.0.x < 0.,
            ..Default::default()
        },
    );
}
pub fn draw_balls(
    room: Query<&crate::util::Room, With<Player2>>,
    balls: Query<(&Position, &crate::util::Room), With<Ball2>>,
    screen: Res<Screen>,
) {
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
    let Ok(room) = room.get_single() else {
        return
    };
    for (position, ball_room) in &balls {
        if ball_room.0 != room.0 {
            continue;
        }
        draw_circ(&screen, position.0.x, position.0.y, BALL_RADIUS, RED);
    }
}

pub fn draw_enemies(
    screen: Res<Screen>,
    enemies: Query<(&Position, &Health, &crate::util::Room), With<Enemy2>>,
    room: Query<&crate::util::Room, With<Player2>>,
) {
    let Ok(drawing_room) = room.get_single() else {
        return
    };
    for (position, health, room) in &enemies {
        if room.0 != drawing_room.0 {
            continue;
        }
        draw_circ(&screen, position.0.x, position.0.y, PLAYER_RADIUS, ORANGE);
        if health == &Health::Dead {
            draw_circ(&screen, position.0.x, position.0.y, PLAYER_RADIUS / 2., RED);
        }
        // TODO:
        // if enemy.slash > 0 {
        //     let slash_x = enemy.body.sight.x * PLAYER_RADIUS + enemy.body.position.x;
        //     let slash_y = enemy.body.sight.y * PLAYER_RADIUS + enemy.body.position.y;
        //     let slash_x_end = enemy.body.sight.x * SLASH_LEN + slash_x;
        //     let slash_y_end = enemy.body.sight.y * SLASH_LEN + slash_y;
        //     draw_lin(
        //         screen,
        //         slash_x,
        //         slash_y,
        //         slash_x_end,
        //         slash_y_end,
        //         0.05,
        //         GRAY,
        //     );
        // }
    }
}

pub fn draw_doors(
    screen: Res<Screen>,
    doors: Query<&Door>,
    player: Query<(&crate::util::Room, &Health), With<Player2>>,
) {
    let Ok((drawing_room, health)) = player.get_single() else {
        return
    };
    draw_rect(
        &screen,
        0.,
        0.,
        RATIO_W_H,
        1.,
        if health == &Health::Full { GRAY } else { RED },
    );
    draw_rect(
        &screen,
        WALL_SIZE,
        WALL_SIZE,
        RATIO_W_H - 2. * WALL_SIZE,
        1. - 2. * WALL_SIZE,
        WHITE,
    );
    for door in &doors {
        if door.from.0 != drawing_room.0 {
            continue;
        }

        let (x, y, w, h) = match door.direction {
            crate::util::Direction::North => (RATIO_W_H / 2. - 0.15, 0.0, 0.3, WALL_SIZE),
            crate::util::Direction::South => {
                (RATIO_W_H / 2. - 0.15, 1.0 - WALL_SIZE, 0.3, WALL_SIZE)
            }
            crate::util::Direction::East => (RATIO_W_H - WALL_SIZE, 0.5 - 0.15, WALL_SIZE, 0.3),
            crate::util::Direction::West => (0.0, 0.5 - 0.15, WALL_SIZE, 0.3),
        };
        draw_rect(&screen, x, y, w, h, BROWN);
    }
}

pub fn draw_phrase(
    phrases: Query<(&Phrase, &crate::util::Room, &Position, &Form)>,
    screen: Res<Screen>,
    room: Query<&crate::util::Room, With<Player2>>,
) {
    let Ok(drawing_room) = room.get_single() else {
        return
    };
    for (phrase, room, position, form) in &phrases {
        if room.0 != drawing_room.0 {
            continue;
        }

        let (lines, max_len) = get_lines(&screen, 8. * PLAYER_RADIUS, 0.04, &phrase.text);
        let start = position.0.y - (lines.len() as f32 * 0.02) - form.y_r() - 0.02;
        draw_rect(
            &screen,
            position.0.x,
            start - 0.02,
            0.04 + max_len,
            lines.len() as f32 * 0.02 + 0.04,
            BLACK,
        );
        for (n, line) in lines.into_iter().enumerate() {
            draw_txt(
                &screen,
                line,
                position.0.x + 0.02,
                start + (0.02 * (n + 1) as f32),
                0.04,
                WHITE,
            );
        }
    }
}

pub fn push_room(
    rooms: &mut Vec<(u8, Vec<EnemyBundle>, Vec<Door>)>,
    room: &RoomConfig,
    room_map: &HashMap<&RoomConfig, Vec<(Direction, &RoomConfig)>>,
) -> Option<usize> {
    let mut connected_rooms = HashMap::new();
    for (direction, room) in room_map.get(room).unwrap().iter().copied() {
        if connected_rooms.insert(direction, room).is_some() {
            return None;
        }
    }
    rooms.push((
        room.id,
        (0..room.enemies)
            .map(|_| {
                let position = Vec2 {
                    x: gen_range(RATIO_W_H / 3.0, 2. * RATIO_W_H / 3.),
                    y: gen_range(0.25, 0.75),
                };
                EnemyBundle {
                    enemy: Enemy2,
                    body: Body2 {
                        position: Position(position),
                        form: Form::Circle {
                            radius: PLAYER_RADIUS,
                        },
                        sight: Sight(Vec2::new(1., 0.)),
                        speed: Speed::default(),
                        room: Room(room.id),
                    },
                    reload: Reload::default(),
                    state: EnemyState::Idle,
                    post: Post(position),
                    health: Health::Low,
                }
            })
            .collect(),
        Vec::new(),
    ));
    let room_pos = rooms.len() - 1;
    rooms[room_pos].2 = connected_rooms
        .into_iter()
        .map(|(direction, room)| {
            let to = rooms
                .iter()
                .position(|r| r.0 == room.id)
                .or_else(|| push_room(rooms, room, room_map))
                .map(|room| Room(room as u8))?;
            Some(Door {
                direction,
                from: Room(room_pos as u8),
                to,
            })
        })
        .collect::<Option<_>>()?;
    Some(room_pos)
}
