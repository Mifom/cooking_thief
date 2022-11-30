use std::{cmp::Ordering, collections::HashMap, f32::consts::FRAC_PI_2, hash::Hash};

use macroquad::{audio::play_sound_once, prelude::*, rand::gen_range};
use serde::Deserialize;

use crate::{
    assets::Assets,
    graphics::{draw_centered_txt, draw_circ, draw_rect, draw_txt, get_lines, Screen},
    RATIO_W_H,
};

pub const BALL_SPEED: f32 = 1.75;
pub const PLAYER_RADIUS: f32 = 0.025;
pub const BALL_RADIUS: f32 = 0.01;
pub const WALL_SIZE: f32 = 0.02;

pub const SPEED_STEPS: i32 = 10;
pub const PLAYER_MAX_SPEED: f32 = 0.65;
pub const PLAYER_RELOAD: f32 = 0.5;
pub const SLASH_LEN: f32 = 0.02;
pub const HEAL_TIME: f32 = 5.;

#[derive(Clone)]
pub struct Velocity(pub Vec2);

#[derive(Default, Clone)]
pub struct Speed {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct Position(pub Vec2);

impl Position {
    pub fn move_to(&self, position: Vec2) -> (i32, i32) {
        let mut move_direction = (0, 0);
        if self.0.distance(position) < 1.5 * PLAYER_RADIUS {
            return move_direction;
        }
        if self.0.y > position.y {
            move_direction.1 = -1;
        } else if self.0.y < position.y {
            move_direction.1 = 1;
        }
        if self.0.x > position.x {
            move_direction.0 = -1;
        } else if self.0.x < position.x {
            move_direction.0 = 1;
        }
        move_direction
    }
}

#[derive(Clone)]
pub struct Sight(pub Vec2);

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Health {
    Full,
    Low,
    Dead,
}

impl Health {
    pub fn decrease(&mut self) {
        *self = match self {
            Self::Full => Self::Low,
            Self::Low | Self::Dead => Self::Dead,
        };
    }
}

#[derive(Clone)]
pub struct Phrase {
    pub text: String,
    pub time: f32,
}

#[derive(Default, Clone)]
pub struct Reload(pub f32);

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Room(pub u8);

#[derive(Clone)]
pub struct Body {
    pub position: Position,
    pub form: Form,
    pub sight: Sight,
    pub speed: Speed,
    pub room: Room,
    pub phrase: Option<Phrase>,
}

#[derive(Clone)]
pub struct Player {
    pub body: Body,
    pub reload: Reload,
    pub health: Health,
    pub item: Item,
    pub visible: bool,
    pub heal_time: f32,
}

#[derive(Clone, serde::Deserialize, PartialEq, Eq)]
pub enum Item {
    Sword,
    Key,
    Vegetable { name: String, idx: usize },
}

#[derive(Clone)]
pub struct ItemCrate {
    pub item: Item,
    pub position: Position,
    pub form: Form,
    pub room: Room,
}

impl ItemCrate {
    pub fn new(item: Item, position: Position, room: Room) -> Self {
        Self {
            item,
            position,
            room,
            form: Form::Rect {
                width: 1.5 * PLAYER_RADIUS,
                height: 1.5 * PLAYER_RADIUS,
            },
        }
    }
}

impl Item {
    pub fn rect(&self) -> Rect {
        match self {
            Self::Sword => Rect::new(80., 20., 100., 120.),
            Self::Key => Rect::new(200., 20., 60., 60.),
            Self::Vegetable { idx, .. } => Rect::new(20. + (*idx as f32 * 60.), 150., 50., 50.),
        }
    }
    pub fn name(&self) -> String {
        match self {
            Self::Sword => "sword",
            Self::Key => "key",
            Self::Vegetable { name, .. } => name,
        }
        .to_owned()
    }
}

#[derive(Default, Clone)]
pub enum EnemyState {
    Fight(Vec2, Form),
    LastSeen(Vec2, f32),
    #[default]
    Idle,
}
#[derive(Clone)]
pub struct Post(pub Vec2);

#[derive(Clone)]
pub struct Enemy {
    pub body: Body,
    pub reload: Reload,
    pub state: EnemyState,
    pub post: Post,
    pub health: Health,
}

#[derive(Clone)]
pub struct Ball {
    pub position: Position,
    pub velocity: Velocity,
    pub room: Room,
    pub item: Item,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub const fn inverse(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

#[derive(Clone)]
pub struct Door {
    pub direction: Direction,
    pub from: Room,
    pub to: Room,
    pub closed: bool,
    pub entrance: bool,
    pub playing: f32,
}

impl Door {
    pub fn new(from: Room, to: Room, direction: Direction, closed: bool, entrance: bool) -> Self {
        Self {
            direction,
            from,
            to,
            closed,
            entrance,
            playing: 0.,
        }
    }
    pub fn door_from(&self, from: &Room) -> Option<(Direction, Room)> {
        if from == &self.from {
            Some((self.direction, self.to))
        } else if from == &self.to {
            Some((self.direction.inverse(), self.from))
        } else {
            None
        }
    }
}

impl PartialEq for Door {
    fn eq(&self, other: &Self) -> bool {
        (self.from == other.from && self.to == other.to)
            || (self.from == other.to && self.to == other.from)
    }
}

#[derive(Clone, Copy)]
pub struct MoveAction {
    pub move_direction: (i32, i32),
    pub sight: Vec2,
}

impl Default for MoveAction {
    fn default() -> Self {
        Self {
            move_direction: (0, 0),
            sight: Vec2::new(1., 0.),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Form {
    Circle { radius: f32 },
    Rect { width: f32, height: f32 },
}

impl Form {
    pub const fn x_r(&self) -> f32 {
        match self {
            Self::Circle { radius } => *radius,
            Self::Rect { width, .. } => *width,
        }
    }
    pub const fn y_r(&self) -> f32 {
        match self {
            Self::Circle { radius } => *radius,
            Self::Rect { height, .. } => *height,
        }
    }

    pub fn direction_len(&self, n: Vec2) -> f32 {
        match self {
            Self::Circle { radius } => *radius,
            Self::Rect { width, height } => {
                let n = n.normalize();
                let x_ratio = width / n.x.abs();
                let y_ratio = height / n.y.abs();
                match x_ratio.total_cmp(&y_ratio) {
                    Ordering::Less => x_ratio,
                    _ => y_ratio,
                }
            }
        }
    }
}
#[derive(Deserialize, Clone)]
pub struct LevelConfig {
    pub rooms: Vec<RoomConfig>,
}

#[derive(Clone, Deserialize)]
pub struct RoomConfig {
    pub id: u8,
    pub enter: Option<Direction>,
    pub doors: Vec<DoorConfig>,
    pub items: Option<Vec<Item>>,
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
    #[serde(default)]
    pub closed: bool,
}

pub fn push_room(
    rooms: &mut Vec<(u8, Vec<Enemy>, Vec<ItemCrate>)>,
    room: &RoomConfig,
    room_map: &HashMap<&RoomConfig, Vec<(Direction, &RoomConfig, bool)>>,
) -> Option<usize> {
    let mut connected_rooms = HashMap::new();
    for (direction, room, _) in room_map.get(room).unwrap().iter().copied() {
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
                Enemy {
                    body: Body {
                        position: Position(position),
                        form: Form::Rect {
                            width: PLAYER_RADIUS,
                            height: 1.7 * PLAYER_RADIUS,
                        },
                        sight: Sight(Vec2::new(1., 0.)),
                        speed: Speed::default(),
                        room: Room(room.id),
                        phrase: None,
                    },
                    reload: Reload::default(),
                    state: EnemyState::Idle,
                    post: Post(position),
                    health: Health::Low,
                }
            })
            .collect(),
        room.items
            .as_ref()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|item| {
                ItemCrate::new(
                    item,
                    Position(Vec2 {
                        x: gen_range(RATIO_W_H / 3.0, 2. * RATIO_W_H / 3.),
                        y: gen_range(0.25, 0.75),
                    }),
                    Room(room.id),
                )
            })
            .collect(),
    ));
    let room_pos = rooms.len() - 1;
    connected_rooms.into_values().try_for_each(|room| {
        rooms
            .iter()
            .position(|r| r.0 == room.id)
            .or_else(|| push_room(rooms, room, room_map))
            .map(|_| ())
    })?;
    Some(room_pos)
}

pub struct Level {
    pub level: LevelInner,
    backup: LevelInner,
}

#[derive(Clone)]
pub struct LevelInner {
    pub player: Player,
    enemies: Vec<Enemy>,
    balls: Vec<Ball>,
    doors: Vec<Door>,
    crates: Vec<ItemCrate>,
}

impl Level {
    pub fn load(config: &LevelConfig) -> Self {
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
                                .map(|door| (door.direction, connected, door.closed))
                                .or_else(|| {
                                    connected.doors.iter().find(|door| door.to == room.id).map(
                                        |door| (door.direction.inverse(), connected, door.closed),
                                    )
                                })
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let mut enters: Vec<_> = rooms.iter().filter(|room| room.enter.is_some()).collect();
        let entry_room = match enters.len() {
            1 => enters.pop().unwrap(),
            // 0 => return Err(Error::NoEntry),
            // _ => {
            //     return Err(Error::MoreThanOneEntry(
            //         enters.iter().map(|room| room.id).collect(),
            //     ))
            // }
            _ => panic!("not one enter"),
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
        let mut result_rooms = Vec::with_capacity(rooms.len());
        let current_room = push_room(&mut result_rooms, entry_room, &room_map).unwrap() as u8;
        let player = Player {
            body: Body {
                position: Position(position),
                form: Form::Rect {
                    width: 1.5 * PLAYER_RADIUS,
                    height: 1.5 * PLAYER_RADIUS,
                },
                sight: Sight(Vec2::new(1., 0.)),
                speed: Speed::default(),
                room: Room(current_room),
                phrase: None,
            },
            reload: Reload::default(),
            health: Health::Full,
            item: Item::Sword,
            visible: false,
            heal_time: HEAL_TIME,
        };
        let mut enemies = Vec::new();
        let mut crates = Vec::new();
        for room in result_rooms {
            enemies.extend(room.1);
            crates.extend(room.2);
        }
        let mut doors: Vec<_> = rooms
            .iter()
            .flat_map(|room| room.doors.iter().map(|door| (room.id, door)))
            .map(|(from, door)| {
                Door::new(
                    Room(from),
                    Room(door.to),
                    door.direction,
                    door.closed,
                    false,
                )
            })
            .collect();
        doors.push(Door::new(
            Room(current_room),
            Room(u8::MAX),
            enter,
            false,
            true,
        ));
        let inner = LevelInner {
            player,
            enemies,
            balls: Vec::new(),
            doors,
            crates,
        };
        Self {
            backup: inner.clone(),
            level: inner,
        }
    }
}

fn player_action(
    screen: &Screen,
    player: &mut Player,
    balls: &mut Vec<Ball>,
    assets: &Assets,
    dt: f32,
) -> MoveAction {
    if player.health == Health::Dead {
        player.body.form = Form::Rect {
            width: 1.5 * PLAYER_RADIUS,
            height: 0.9 * PLAYER_RADIUS,
        };
        return MoveAction::default();
    }
    let mut move_direction = (0, 0);
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        move_direction.1 -= 1;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        move_direction.1 += 1;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        move_direction.0 -= 1;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        move_direction.0 += 1;
    }
    let (x_mouse, y_mouse) = {
        let (x_m, y_m) = mouse_position();
        (
            clamp((x_m - screen.x) / screen.height, 0., RATIO_W_H),
            clamp((y_m - screen.y) / screen.height, 0., 1.),
        )
    };
    let sight = Vec2 {
        x: x_mouse - player.body.position.0.x,
        y: y_mouse - player.body.position.0.y,
    }
    .normalize_or_zero();
    let move_action = MoveAction {
        move_direction,
        sight,
    };

    if is_key_pressed(KeyCode::Space) {
        player.body.form = if player.visible {
            player.visible = false;
            Form::Rect {
                width: 1.5 * PLAYER_RADIUS,
                height: 1.5 * PLAYER_RADIUS,
            }
        } else {
            player.visible = true;
            Form::Rect {
                width: PLAYER_RADIUS,
                height: 1.5 * PLAYER_RADIUS,
            }
        };
    }
    if is_mouse_button_down(MouseButton::Left)
        && (player.visible || cfg!(feature = "cheat"))
        && player.reload.0 == 0.
    {
        match player.item {
            Item::Vegetable { .. } => {
                player.reload.0 = PLAYER_RELOAD;
                let position = player.body.position.0 + (move_action.sight * PLAYER_RADIUS);
                balls.push(Ball {
                    position: Position(position),
                    velocity: Velocity(move_action.sight * BALL_SPEED),
                    room: player.body.room,
                    item: player.item.clone(),
                });
                play_sound_once(assets.sounds["throw"]);
            }
            _ => {
                player.body.phrase = Some(Phrase {
                    text: format!("I can't attack with {}", player.item.name()),
                    time: 3.,
                });
            }
        }
    }
    if player.health == Health::Low {
        player.heal_time -= dt;
        if player.heal_time <= 0. {
            player.heal_time = HEAL_TIME;
            player.health = Health::Full;
        }
    }

    move_action
}

fn enemy_action(enemy: &mut Enemy, player: &mut Player, assets: &Assets, dt: f32) -> MoveAction {
    if enemy.health == Health::Dead {
        enemy.body.form = Form::Rect {
            width: 1.7 * PLAYER_RADIUS,
            height: 0.9 * PLAYER_RADIUS,
        };
        return MoveAction::default();
    }
    let diff = enemy.body.position.0 - player.body.position.0;
    let touch_distance = if player.health == Health::Full {
        SLASH_LEN / 2.
    } else {
        SLASH_LEN / 6.
    };
    let player_visible = player.visible
        || diff.length()
            < enemy.body.form.direction_len(diff)
                + player.body.form.direction_len(diff)
                + touch_distance;
    let mut phrase = None;
    enemy.state = if player.health == Health::Dead {
        EnemyState::Idle
    } else if player.body.room == enemy.body.room && player_visible {
        if !matches!(enemy.state, EnemyState::Fight(_, _)) {
            phrase = Some(Phrase {
                text: "Here you are!".to_owned(),
                time: 1.,
            });
        }
        EnemyState::Fight(player.body.position.0, player.body.form)
    } else {
        match enemy.state {
            EnemyState::Fight(position, _) => {
                phrase = Some(Phrase {
                    text: "Where is he?".to_owned(),
                    time: 2.,
                });
                EnemyState::LastSeen(position, dt)
            }
            EnemyState::Idle => EnemyState::Idle,
            EnemyState::LastSeen(position, timer) => {
                let new_timer = timer + dt;
                if new_timer > 5. {
                    phrase = Some(Phrase {
                        text: "Must've been wind".to_owned(),
                        time: 2.,
                    });
                    EnemyState::Idle
                } else {
                    EnemyState::LastSeen(position, new_timer)
                }
            }
        }
    };
    if let Some(phrase) = phrase {
        enemy.body.phrase = Some(phrase);
    }
    let (move_action, slash) = match enemy.state {
        EnemyState::Idle => (
            MoveAction {
                move_direction: enemy.body.position.move_to(enemy.post.0),
                sight: Vec2 { x: 1., y: 0. },
            },
            false,
        ),
        EnemyState::Fight(player_position, player_form) => {
            let diff = player_position - enemy.body.position.0;
            (
                MoveAction {
                    move_direction: enemy.body.position.move_to(player_position),
                    sight: (player_position - enemy.body.position.0).normalize(),
                },
                diff.length()
                    < enemy.body.form.direction_len(diff)
                        + player_form.direction_len(diff)
                        + SLASH_LEN,
            )
        }
        EnemyState::LastSeen(last_position, _) => (
            MoveAction {
                move_direction: enemy.body.position.move_to(last_position),
                sight: last_position - enemy.body.position.0,
            },
            false,
        ),
    };
    if slash && enemy.reload.0 == 0. {
        enemy.reload.0 = PLAYER_RELOAD;
        player.health.decrease();
        play_sound_once(assets.sounds["sword"]);
    }
    enemy.body.form = if enemy.reload.0 < 0.2 {
        Form::Rect {
            width: PLAYER_RADIUS,
            height: 1.7 * PLAYER_RADIUS,
        }
    } else {
        Form::Rect {
            width: 1.15 * PLAYER_RADIUS,
            height: 1.7 * PLAYER_RADIUS,
        }
    };
    move_action
}

fn collide(mut bodies: Vec<&mut Body>, crates: &Vec<ItemCrate>) {
    let mut shifts = HashMap::new();
    for (left_id, left) in bodies.iter().enumerate() {
        for item_crate in crates {
            if left.room != item_crate.room {
                continue;
            }

            let diff = left.position.0 - item_crate.position.0;
            let size = left.form.direction_len(diff) + item_crate.form.direction_len(diff);
            let penetration = size - diff.length();

            if penetration > 0. {
                let shift = diff.normalize() * penetration;
                shifts
                    .entry(left_id)
                    .and_modify(|v| *v += shift)
                    .or_insert_with(|| shift);
            }
        }
        for (right_id, right) in bodies.iter().enumerate() {
            if left_id == right_id || left.room != right.room {
                shifts.entry(left_id).or_default();
                shifts.entry(right_id).or_default();
                continue;
            }

            let diff = left.position.0 - right.position.0;
            let size = left.form.direction_len(diff) + right.form.direction_len(diff);
            let penetration = (size - diff.length()) / 2.;

            if penetration > 0. {
                let shift = diff.normalize() * penetration;
                shifts
                    .entry(left_id)
                    .and_modify(|v| *v += shift)
                    .or_insert_with(|| shift);
                shifts
                    .entry(right_id)
                    .and_modify(|v| *v -= shift)
                    .or_insert_with(|| -shift);
            }
        }
    }
    for (id, body) in bodies.iter_mut().enumerate() {
        body.position.0 += shifts[&id];
        let x_wall = body.form.x_r();
        let y_wall = body.form.y_r();
        body.position.0.x = clamp(
            body.position.0.x,
            WALL_SIZE + x_wall,
            RATIO_W_H - WALL_SIZE - x_wall,
        );
        body.position.0.y = clamp(
            body.position.0.y,
            WALL_SIZE + y_wall,
            1. - WALL_SIZE - y_wall,
        );
    }
}

fn use_door(player: &mut Player, door: &mut Door, enemies: &Vec<Enemy>, assets: &Assets) -> bool {
    if let Some((direction, to)) = door.door_from(&player.body.room) {
        let (x_range, y_range) = match direction {
            Direction::North => (
                (RATIO_W_H / 2. - 0.15..=RATIO_W_H / 2. + 0.15),
                (0.0..=WALL_SIZE + 0.05),
            ),
            Direction::South => (
                (RATIO_W_H / 2. - 0.15..=RATIO_W_H / 2. + 0.15),
                ((1.0 - WALL_SIZE - 0.05)..=1.0),
            ),
            Direction::East => (((RATIO_W_H - WALL_SIZE - 0.05)..=RATIO_W_H), (0.35..=0.65)),
            Direction::West => ((0.0..=(WALL_SIZE + 0.05)), (0.35..=0.65)),
        };
        if x_range.contains(&player.body.position.0.x)
            && y_range.contains(&player.body.position.0.y)
        {
            if door.entrance {
                if enemies.iter().any(|enemy| enemy.health != Health::Dead) {
                    player.body.phrase = Some(Phrase {
                        text: "The guards are still on guard".to_owned(),
                        time: 2.,
                    });
                } else if player.item != Item::Sword {
                    player.body.phrase = Some(Phrase {
                        text: "I can't leave sword here".to_owned(),
                        time: 2.,
                    });
                } else {
                    return true;
                }
                if door.playing == 0. {
                    door.playing = 1.;
                    play_sound_once(assets.sounds["door_locked"]);
                }
                return false;
            }
            if door.closed && player.item != Item::Key {
                if door.playing == 0. {
                    door.playing = 1.;
                    play_sound_once(assets.sounds["door_locked"]);
                }
                player.body.phrase = Some(Phrase {
                    text: "It's locked".to_owned(),
                    time: 1.,
                });
            } else {
                if door.closed {
                    play_sound_once(assets.sounds["door_unlock"]);
                }
                door.closed = false;
                match direction {
                    Direction::North | Direction::South => {
                        player.body.position.0.y = clamp(1. - player.body.position.0.y, 0.1, 0.9);
                    }
                    Direction::East | Direction::West => {
                        player.body.position.0.x =
                            clamp(RATIO_W_H - player.body.position.0.x, 0.1, RATIO_W_H - 0.1);
                    }
                }
                player.body.room = to;
            }
        }
    }
    false
}

fn swap_items(item_crate: &mut ItemCrate, player: &mut Player, assets: &Assets) -> bool {
    if item_crate.room.0 != player.body.room.0 {
        return false;
    }
    let diff = item_crate.position.0 - player.body.position.0;
    if is_key_pressed(KeyCode::E)
        && diff.length()
            <= player.body.form.direction_len(diff) + item_crate.form.direction_len(diff) + 0.02
    {
        (player.item, item_crate.item) = (item_crate.item.clone(), player.item.clone());
        play_sound_once(assets.sounds["item"]);
        true
    } else {
        false
    }
}

pub fn update_level(level: &mut Level, screen: &Screen, assets: &Assets, dt: f32) -> bool {
    let Level { level, backup } = level;
    let mut next = false;
    let player_action = player_action(screen, &mut level.player, &mut level.balls, assets, dt);
    level
        .enemies
        .iter_mut()
        .map(|enemy| {
            (
                enemy_action(enemy, &mut level.player, assets, dt),
                &mut enemy.body,
            )
        })
        .collect::<Vec<_>>()
        .into_iter()
        .chain(std::iter::once((player_action, &mut level.player.body)))
        .for_each(|(move_action, body)| {
            // move body
            body.sight.0 = move_action.sight;
            body.speed.x += 2 * move_action.move_direction.0;
            body.speed.y += 2 * move_action.move_direction.1;

            match body.speed.x.cmp(&0) {
                std::cmp::Ordering::Less => body.speed.x += 1,
                std::cmp::Ordering::Greater => body.speed.x -= 1,
                _ => {}
            }
            body.speed.x = clamp(body.speed.x, -SPEED_STEPS, SPEED_STEPS);
            match body.speed.y.cmp(&0) {
                std::cmp::Ordering::Less => body.speed.y += 1,
                std::cmp::Ordering::Greater => body.speed.y -= 1,
                _ => {}
            }
            body.speed.y = clamp(body.speed.y, -SPEED_STEPS, SPEED_STEPS);
            body.position.0.x +=
                PLAYER_MAX_SPEED * (body.speed.x as f32) / (SPEED_STEPS as f32) * dt;
            body.position.0.y +=
                PLAYER_MAX_SPEED * (body.speed.y as f32) / (SPEED_STEPS as f32) * dt;
        });
    collide(
        level
            .enemies
            .iter_mut()
            .map(|enemy| &mut enemy.body)
            .chain(std::iter::once(&mut level.player.body))
            .collect(),
        &level.crates,
    );
    if level
        .doors
        .iter_mut()
        .map(|door| {
            door.playing = clamp(door.playing - dt, 0., door.playing);
            door
        })
        .any(|door| use_door(&mut level.player, door, &level.enemies, assets))
    {
        next = true;
    }
    level
        .enemies
        .iter_mut()
        .map(|enemy| &mut enemy.reload)
        .chain(std::iter::once(&mut level.player.reload))
        .for_each(|reload| {
            reload.0 = clamp(reload.0 - dt, 0., reload.0);
        });
    level.balls = level
        .balls
        .iter_mut()
        .map(|ball| {
            ball.position.0 += ball.velocity.0 * dt;
            for enemy in &mut level.enemies {
                if ball.room != enemy.body.room || enemy.health == Health::Dead {
                    continue;
                }
                let diff = ball.position.0 - enemy.body.position.0;
                if diff.length() < BALL_RADIUS + enemy.body.form.direction_len(diff) {
                    enemy.health.decrease();
                    return None;
                }
            }
            if ball.position.0.x < WALL_SIZE + BALL_RADIUS
                || ball.position.0.x > RATIO_W_H - WALL_SIZE - BALL_RADIUS
                || ball.position.0.y < WALL_SIZE + BALL_RADIUS
                || ball.position.0.y > 1. - WALL_SIZE - BALL_RADIUS
            {
                return None;
            }

            Some(ball.clone())
        })
        .filter_map(|ball| {
            if ball.is_none() {
                play_sound_once(assets.sounds["splat"]);
            }
            ball
        })
        .collect();

    level
        .enemies
        .iter_mut()
        .map(|enemy| (&mut enemy.body.phrase, &enemy.health))
        .chain(std::iter::once((
            &mut level.player.body.phrase,
            &level.player.health,
        )))
        .for_each(|(phrase, health)| {
            if let Some(phrase_inner) = phrase {
                phrase_inner.time -= dt;
                if phrase_inner.time <= 0. || health == &Health::Dead {
                    *phrase = None;
                }
            }
        });

    if level
        .crates
        .iter_mut()
        .any(|item_crate| swap_items(item_crate, &mut level.player, assets))
    {
        *backup = level.clone();
    }

    if level.player.health == Health::Dead && is_key_pressed(KeyCode::R) {
        *level = backup.clone();
    }
    next
}

fn draw_doors(screen: &Screen, player: &Player, doors: &Vec<Door>, assets: &Assets) {
    draw_texture_ex(
        assets.images["level_back"],
        screen.x,
        screen.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2::new(screen.width, screen.height)),
            ..Default::default()
        },
    );
    for door in doors {
        if let Some((direction, _)) = door.door_from(&player.body.room) {
            let rect_x = if door.entrance {
                42.
            } else if door.closed {
                21.
            } else {
                0.
            };

            let (x, y, rotation_multiplier) = match direction {
                Direction::North => (RATIO_W_H / 2., WALL_SIZE / 2. - 0.15, 1.),
                Direction::South => (RATIO_W_H / 2., 1.0 - WALL_SIZE / 2. - 0.15, 1.),
                Direction::East => (RATIO_W_H - WALL_SIZE, 0.5 - 0.15, 0.),
                Direction::West => (0.0, 0.5 - 0.15, 0.),
            };
            draw_texture_ex(
                assets.images["doors"],
                x * screen.height + screen.x,
                y * screen.height + screen.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(WALL_SIZE * screen.height, 0.3 * screen.height)),
                    source: Some(Rect {
                        x: rect_x,
                        y: 0.,
                        w: 20.,
                        h: 324.,
                    }),
                    rotation: rotation_multiplier * FRAC_PI_2,
                    ..Default::default()
                },
            )
        }
    }
}

pub fn draw_level(level: &Level, assets: &Assets, screen: &Screen) {
    let Level { level, .. } = level;
    draw_doors(screen, &level.player, &level.doors, assets);
    // Player
    draw_texture_ex(
        assets.images["player"],
        (level.player.body.position.0.x - level.player.body.form.x_r()) * screen.height + screen.x,
        (level.player.body.position.0.y - level.player.body.form.y_r()) * screen.height + screen.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2 {
                x: 2. * level.player.body.form.x_r() * screen.height,
                y: 2. * level.player.body.form.y_r() * screen.height,
            }),
            source: Some(if level.player.health == Health::Dead {
                Rect {
                    x: 280.,
                    y: 10.,
                    w: 150.,
                    h: 90.,
                }
            } else if level.player.visible {
                Rect {
                    x: 10.,
                    y: 10.,
                    w: 100.,
                    h: 150.,
                }
            } else {
                Rect {
                    x: 120.,
                    y: 10.,
                    w: 150.,
                    h: 150.,
                }
            }),
            flip_x: level.player.body.sight.0.x < 0.,
            ..Default::default()
        },
    );

    // Balls
    for ball in &level.balls {
        if ball.room != level.player.body.room {
            continue;
        }
        draw_texture_ex(
            assets.images["items"],
            (ball.position.0.x - BALL_RADIUS) * screen.height + screen.x,
            (ball.position.0.y - BALL_RADIUS) * screen.height + screen.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: 2. * BALL_RADIUS * screen.height,
                    y: 2. * BALL_RADIUS * screen.height,
                }),
                source: Some(ball.item.rect()),
                ..Default::default()
            },
        );
    }

    // Enemies
    for enemy in &level.enemies {
        if enemy.body.room != level.player.body.room {
            continue;
        }
        draw_texture_ex(
            assets.images["enemy"],
            (enemy.body.position.0.x - enemy.body.form.x_r()) * screen.height + screen.x,
            (enemy.body.position.0.y - enemy.body.form.y_r()) * screen.height + screen.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: 2. * enemy.body.form.x_r() * screen.height,
                    y: 2. * enemy.body.form.y_r() * screen.height,
                }),
                source: Some(if enemy.health == Health::Dead {
                    Rect {
                        x: 250.,
                        y: 10.,
                        w: 170.,
                        h: 90.,
                    }
                } else if enemy.reload.0 < 0.2 {
                    Rect {
                        x: 10.,
                        y: 10.,
                        w: 100.,
                        h: 170.,
                    }
                } else {
                    Rect {
                        x: 120.,
                        y: 10.,
                        w: 115.,
                        h: 170.,
                    }
                }),
                flip_x: enemy.body.sight.0.x < 0.,
                ..Default::default()
            },
        );
        if enemy.health == Health::Dead {
            draw_circ(
                &screen,
                enemy.body.position.0.x,
                enemy.body.position.0.y,
                PLAYER_RADIUS / 2.,
                RED,
            );
        }
    }

    // Crates
    for item_crate in &level.crates {
        if item_crate.room != level.player.body.room {
            continue;
        }
        draw_texture_ex(
            assets.images["crate"],
            (item_crate.position.0.x - item_crate.form.x_r()) * screen.height + screen.x,
            (item_crate.position.0.y - item_crate.form.y_r()) * screen.height + screen.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    2. * item_crate.form.x_r() * screen.height,
                    2. * item_crate.form.y_r() * screen.height,
                )),
                ..Default::default()
            },
        );
        draw_texture_ex(
            assets.images["items"],
            (item_crate.position.0.x - 1.5 * BALL_RADIUS) * screen.height + screen.x,
            (item_crate.position.0.y - 1.5 * BALL_RADIUS) * screen.height + screen.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: 3. * BALL_RADIUS * screen.height,
                    y: 3. * BALL_RADIUS * screen.height,
                }),
                source: Some(item_crate.item.rect()),
                ..Default::default()
            },
        );
        let diff = item_crate.position.0 - level.player.body.position.0;
        if diff.length()
            <= level.player.body.form.direction_len(diff)
                + item_crate.form.direction_len(diff)
                + 0.02
        {
            draw_txt(
                &screen,
                "E to use",
                item_crate.position.0.x,
                item_crate.position.0.y - item_crate.form.y_r() - 0.02,
                0.08,
                GREEN,
            );
        }
    }

    // Phrases
    for body in level
        .enemies
        .iter()
        .map(|enemy| &enemy.body)
        .chain(std::iter::once(&level.player.body))
    {
        if body.room != level.player.body.room {
            continue;
        }
        let Some(phrase) = &body.phrase else {
                    continue;
                };

        let (lines, max_len) = get_lines(&screen, 8. * PLAYER_RADIUS, 0.04, &phrase.text);
        let start = body.position.0.y - (lines.len() as f32 * 0.02) - body.form.y_r() - 0.02;
        draw_rect(
            &screen,
            body.position.0.x,
            start - 0.02,
            0.04 + max_len,
            lines.len() as f32 * 0.02 + 0.04,
            BLACK,
        );
        for (n, line) in lines.into_iter().enumerate() {
            draw_txt(
                &screen,
                line,
                body.position.0.x + 0.02,
                start + (0.02 * (n + 1) as f32),
                0.04,
                WHITE,
            );
        }
    }

    if level.player.health == Health::Low {
        draw_texture_ex(
            assets.images["blood"],
            screen.x,
            screen.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen.width, screen.height)),
                ..Default::default()
            },
        );
    } else if level.player.health == Health::Dead {
        draw_rect(
            &screen,
            0.,
            0.,
            RATIO_W_H,
            1.,
            Color::from_rgba(128, 0, 0, 128),
        );
        draw_centered_txt(&screen, "You're dead. Press R to continue", 0.5, 0.1, WHITE);
    }
}
