#![allow(clippy::type_complexity)] // To suppress clippy warnings on query types
use std::{cmp::Ordering, collections::HashMap};

use bevy_ecs::{
    prelude::{Bundle, Component, Entity, EventReader, EventWriter},
    query::{Or, With, Without},
    system::{Commands, Query, Res, ResMut, Resource},
};
use macroquad::{prelude::*, rand::gen_range};
use serde::Deserialize;

use crate::{
    graphics::{draw_centered_txt, draw_rect, Screen},
    level::push_room,
    scene::Scene,
};

pub const RATIO_W_H: f32 = 16. / 9.;

pub const BALL_SPEED: f32 = 1.75;
pub const PLAYER_RADIUS: f32 = 0.025;
pub const BALL_RADIUS: f32 = 0.01;
pub const WALL_SIZE: f32 = 0.02;

pub const SPEED_STEPS: i32 = 10;
pub const PLAYER_MAX_SPEED: f32 = 0.65;
pub const PLAYER_RELOAD: f32 = 0.5;
pub const SLASH_LEN: f32 = 0.02;

#[derive(Resource, Debug)]
pub enum StateChange {
    Next,
    Restart,
}

#[derive(Resource, Default)]
pub struct Time {
    pub time: f32,
    pub dt: f32,
}

impl Time {
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.dt = dt;
    }
}

#[derive(Component)]
pub struct Velocity(Vec2);

#[derive(Component, Default)]
pub struct Speed {
    x: i32,
    y: i32,
}

#[derive(Component)]
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

#[derive(Component)]
pub struct Sight(pub Vec2);

#[derive(Component)]
pub struct Visible;

#[derive(Component, PartialEq, Eq, Debug)]
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

#[derive(Component)]
pub struct Phrase {
    pub text: String,
    pub time: f32,
}

#[derive(Component, Default)]
pub struct Reload(f32);

#[derive(Component, PartialEq, Eq, Clone, Copy)]
pub struct Room(pub u8);

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct Body {
    pub position: Position,
    pub form: Form,
    pub sight: Sight,
    pub speed: Speed,
    pub room: Room,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    body: Body,
    reload: Reload,
    health: Health,
    item: Item,
}

#[derive(Component, Default)]
pub enum EnemyState {
    Fight(Vec2, Form),
    LastSeen(Vec2, f32),
    #[default]
    Idle,
}
#[derive(Component)]
pub struct Post(pub Vec2);

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub body: Body,
    pub reload: Reload,
    // pub slash: i8,
    pub state: EnemyState,
    pub post: Post,
    pub health: Health,
}

#[derive(Component)]
pub struct Ball2;

#[derive(Bundle)]
pub struct BallBundle {
    ball: Ball2,
    position: Position,
    velocity: Velocity,
    room: Room,
    item: Item,
}

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Deserialize)]
pub enum Direction {
    North,
    South,
    East,
    West,
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

#[derive(Component)]
pub struct Door {
    direction: Direction,
    from: Room,
    to: Room,
    pub closed: bool,
}

impl Door {
    pub fn new(from: Room, to: Room, direction: Direction, closed: bool) -> Self {
        Self {
            direction,
            from,
            to,
            closed,
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

#[derive(Component)]
pub struct Entrance;

#[derive(Clone, Copy)]
pub struct MoveAction {
    pub move_direction: (i32, i32),
    pub sight: Vec2,
}

#[derive(Clone, Copy, Component)]
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

pub fn player_action(
    mut commands: Commands,
    screen: Res<Screen>,
    mut player: Query<
        (
            Entity,
            &Position,
            Option<&Visible>,
            &Item,
            &mut Form,
            &mut Reload,
            &Room,
            &Health,
        ),
        With<Player>,
    >,
    mut moves: EventWriter<(Entity, MoveAction)>,
) {
    let Ok((player_id, position, visible, item, mut form, mut reload, room, health)) =
        player.get_single_mut() else {
        return;
    };
    if health == &Health::Dead {
        return;
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
        x: x_mouse - position.0.x,
        y: y_mouse - position.0.y,
    }
    .normalize_or_zero();
    let move_action = MoveAction {
        move_direction,
        sight,
    };

    moves.send((player_id, move_action));

    if is_key_pressed(KeyCode::Space) {
        *form = if visible.is_some() {
            commands.entity(player_id).remove::<Visible>();
            Form::Rect {
                width: 1.5 * PLAYER_RADIUS,
                height: 1.5 * PLAYER_RADIUS,
            }
        } else {
            commands.entity(player_id).insert(Visible);
            Form::Rect {
                width: PLAYER_RADIUS,
                height: 1.5 * PLAYER_RADIUS,
            }
        };
    }
    if is_mouse_button_down(MouseButton::Left) && visible.is_some() && reload.0 == 0. {
        match item {
            Item::Vegetable { .. } => {
                reload.0 = PLAYER_RELOAD;
                let position = position.0 + (move_action.sight * PLAYER_RADIUS);
                commands.spawn(BallBundle {
                    position: Position(position),
                    ball: Ball2,
                    velocity: Velocity(move_action.sight * BALL_SPEED),
                    room: *room,
                    item: item.clone(),
                });
            }
            _ => {
                commands.entity(player_id).insert(Phrase {
                    text: format!("I can't attack with {}", item.name()),
                    time: 3.,
                });
            }
        }
    }
}

pub fn enemies_actions(
    time: Res<Time>,
    mut commands: Commands,
    mut moves: EventWriter<(Entity, MoveAction)>,
    mut enemies: Query<
        (
            Entity,
            &Position,
            &Form,
            &mut Reload,
            &mut EnemyState,
            &Post,
            &Health,
            &Room,
        ),
        (With<Enemy>, Without<Player>),
    >,
    mut player: Query<
        (&Position, &Form, &mut Health, Option<&Visible>, &Room),
        (With<Player>, Without<Enemy>),
    >,
) {
    let Ok((player_position, player_form, mut player_health, player_visible, player_room) )=
        player.get_single_mut() else {
        return;
    };
    for (enemy_id, position, form, mut reload, mut state, post, health, room) in &mut enemies {
        if matches!(health, Health::Dead) {
            continue;
        }
        let player_visible = player_room == room
            && (player_visible.is_some()
                || position.0.distance(player_position.0) < 2. * PLAYER_RADIUS + SLASH_LEN / 2.);
        let mut phrase = None;
        *state = if player_visible {
            if !matches!(*state, EnemyState::Fight(_, _)) {
                phrase = Some(Phrase {
                    text: "Here you are!".to_owned(),
                    time: 1.,
                });
            }
            EnemyState::Fight(player_position.0, *player_form)
        } else {
            match *state {
                EnemyState::Fight(position, _) => {
                    phrase = Some(Phrase {
                        text: "Where is he?".to_owned(),
                        time: 2.,
                    });
                    EnemyState::LastSeen(position, time.dt)
                }
                EnemyState::Idle => EnemyState::Idle,
                EnemyState::LastSeen(position, timer) => {
                    let new_timer = timer + time.dt;
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
            if let Some(mut entity) = commands.get_entity(enemy_id) {
                entity.insert(phrase);
            }
        }
        let (move_action, slash) = match *state {
            EnemyState::Idle => (
                MoveAction {
                    move_direction: position.move_to(post.0),
                    sight: Vec2 { x: 1., y: 0. },
                },
                false,
            ),
            EnemyState::Fight(player_position, player_form) => {
                let diff = player_position - position.0;
                (
                    MoveAction {
                        move_direction: position.move_to(player_position),
                        sight: (player_position - position.0).normalize(),
                    },
                    diff.length()
                        < form.direction_len(diff) + player_form.direction_len(diff) + SLASH_LEN,
                )
            }
            EnemyState::LastSeen(last_position, _) => (
                MoveAction {
                    move_direction: position.move_to(last_position),
                    sight: last_position - position.0,
                },
                false,
            ),
        };
        moves.send((enemy_id, move_action));
        if slash && reload.0 == 0. {
            reload.0 = PLAYER_RELOAD;
            player_health.decrease();
        }
    }
}
pub fn use_doors(
    mut doors: Query<(&mut Door, Option<&Entrance>)>,
    mut player: Query<(Entity, &mut Position, &mut Room, &Item), With<Player>>,
    enemies: Query<&Health, With<Enemy>>,
    mut commands: Commands,
) {
    let Ok((player_id, mut position, mut room, item )) = player.get_single_mut() else {
        return;
    };
    for (mut door, entrance) in doors.iter_mut() {
        if let Some((direction, to)) = door.door_from(&room) {
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
            if x_range.contains(&position.0.x) && y_range.contains(&position.0.y) {
                if entrance.is_some() {
                    if enemies.iter().any(|health| health != &Health::Dead) {
                        commands.entity(player_id).insert(Phrase {
                            text: "The guards are still on guard".to_owned(),
                            time: 2.,
                        });
                    } else if item != &Item::Sword {
                        commands.entity(player_id).insert(Phrase {
                            text: "I can't leave sword here".to_owned(),
                            time: 2.,
                        });
                    } else {
                        commands.insert_resource(StateChange::Next);
                    }
                    return;
                }
                if door.closed && item != &Item::Key {
                    commands.entity(player_id).insert(Phrase {
                        text: "It's locked".to_owned(),
                        time: 1.,
                    });
                } else {
                    door.closed = false;
                    match direction {
                        Direction::North | Direction::South => {
                            position.0.y = clamp(1. - position.0.y, 0.1, 0.9);
                        }
                        Direction::East | Direction::West => {
                            position.0.x = clamp(RATIO_W_H - position.0.x, 0.1, RATIO_W_H - 0.1);
                        }
                    }
                    *room = to;
                }
            }
        }
    }
}

pub fn move_body(
    mut bodies: Query<(Entity, &mut Sight, &mut Position, &mut Speed)>,
    time: Res<Time>,
    mut actions: EventReader<(Entity, MoveAction)>,
) {
    for (action_entity, move_action) in actions.iter() {
        for (entity, mut sight, mut position, mut speed) in &mut bodies {
            if *action_entity == entity {
                sight.0 = move_action.sight;
                speed.x += 2 * move_action.move_direction.0;
                speed.y += 2 * move_action.move_direction.1;

                match speed.x.cmp(&0) {
                    std::cmp::Ordering::Less => speed.x += 1,
                    std::cmp::Ordering::Greater => speed.x -= 1,
                    _ => {}
                }
                speed.x = clamp(speed.x, -SPEED_STEPS, SPEED_STEPS);
                match speed.y.cmp(&0) {
                    std::cmp::Ordering::Less => speed.y += 1,
                    std::cmp::Ordering::Greater => speed.y -= 1,
                    _ => {}
                }
                speed.y = clamp(speed.y, -SPEED_STEPS, SPEED_STEPS);
                position.0.x +=
                    PLAYER_MAX_SPEED * (speed.x as f32) / (SPEED_STEPS as f32) * time.dt;
                position.0.y +=
                    PLAYER_MAX_SPEED * (speed.y as f32) / (SPEED_STEPS as f32) * time.dt;

                break;
            }
        }
    }
    actions.clear();
}

pub fn collide(mut bodies: Query<(Entity, &mut Position, &Form, &Room)>) {
    let mut shifts = HashMap::new();
    for (left_id, Position(left_position), left_form, left_room) in bodies.iter() {
        for (right_id, Position(right_position), right_form, right_room) in bodies.iter() {
            if left_id == right_id || left_room != right_room {
                shifts.entry(left_id).or_default();
                shifts.entry(right_id).or_default();
                continue;
            }

            let diff = *left_position - *right_position;
            let size = left_form.direction_len(diff) + right_form.direction_len(diff);
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
    for (entity, mut position, form, _) in &mut bodies {
        position.0 += shifts[&entity];
        let x_wall = form.x_r();
        let y_wall = form.y_r();
        position.0.x = clamp(
            position.0.x,
            WALL_SIZE + x_wall,
            RATIO_W_H - WALL_SIZE - x_wall,
        );
        position.0.y = clamp(position.0.y, WALL_SIZE + y_wall, 1. - WALL_SIZE - y_wall);
    }
}

pub fn update_reload(time: Res<Time>, mut reloads: Query<&mut Reload>) {
    for mut reload in &mut reloads {
        reload.0 = clamp(reload.0 - time.dt, 0., reload.0);
    }
}

pub fn update_phrase(
    mut commands: Commands,
    time: Res<Time>,
    mut phrases: Query<(Entity, &mut Phrase, &Health)>,
) {
    for (entity, mut phrase, health) in &mut phrases {
        phrase.time -= time.dt;
        if phrase.time <= 0. || health == &Health::Dead {
            commands.entity(entity).remove::<Phrase>();
        }
    }
}

pub fn update_balls(time: Res<Time>, mut balls: Query<(&mut Position, &Velocity), With<Ball2>>) {
    for (mut position, Velocity(velocity)) in &mut balls {
        position.0 += *velocity * time.dt;
    }
}

pub fn collide_balls(
    mut commands: Commands,
    balls: Query<(Entity, &Position, &Room), With<Ball2>>,
    mut enemies: Query<(&Position, &Form, &mut Health, &Room), With<Enemy>>,
) {
    'outer: for (ball_id, Position(ball_position), Room(ball_room)) in balls.into_iter() {
        for (Position(enemy_position), enemy_form, mut enemy_health, Room(enemy_room)) in
            &mut enemies
        {
            if ball_room != enemy_room {
                continue;
            }
            let diff = *ball_position - *enemy_position;
            if diff.length() < BALL_RADIUS + enemy_form.direction_len(diff) {
                commands.entity(ball_id).despawn();
                enemy_health.decrease();
                continue 'outer;
            }
        }

        if ball_position.x < WALL_SIZE + BALL_RADIUS
            || ball_position.x > RATIO_W_H - WALL_SIZE - BALL_RADIUS
            || ball_position.y < WALL_SIZE + BALL_RADIUS
            || ball_position.y > 1. - WALL_SIZE - BALL_RADIUS
        {
            commands.entity(ball_id).despawn();
        }
    }
}
pub fn change_state(
    mut state: ResMut<crate::State>,
    state_change: Option<Res<StateChange>>,
    mut commands: Commands,
    entities: Query<
        Entity,
        Or<(
            With<Enemy>,
            With<Player>,
            With<Ball2>,
            With<Door>,
            With<Crate>,
        )>,
    >,
) {
    if let Some(state_change) = state_change {
        commands.remove_resource::<StateChange>();

        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
        commands.remove_resource::<Scene>();

        match *state_change {
            StateChange::Next => match state.as_ref() {
                crate::State::Scene(num) => *state = crate::State::Battle(*num),
                crate::State::Battle(num) => *state = crate::State::Scene(*num + 1),
            },
            StateChange::Restart => match state.as_ref() {
                crate::State::Scene(num) => *state = crate::State::Scene(*num),
                crate::State::Battle(num) => *state = crate::State::Battle(*num),
            },
        }
    }
}

pub fn load_new_state(
    assets: Res<crate::assets::Assets>,
    state: Res<crate::State>,
    mut commands: Commands,
) {
    if state.is_changed() {
        match state.as_ref() {
            crate::State::Scene(num) => {
                let scene = assets.scenes.get(num).unwrap_or_else(|| panic!("{num}"));
                commands.insert_resource(scene.clone());
            }
            crate::State::Battle(num) => {
                let config = assets.levels.get(num).unwrap();

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
                                            connected
                                                .doors
                                                .iter()
                                                .find(|door| door.to == room.id)
                                                .map(|door| {
                                                    (
                                                        door.direction.inverse(),
                                                        connected,
                                                        door.closed,
                                                    )
                                                })
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
                let current_room =
                    push_room(&mut result_rooms, entry_room, &room_map).unwrap() as u8;
                let player = PlayerBundle {
                    player: Player,
                    body: Body {
                        position: Position(position),
                        form: Form::Rect {
                            width: 1.5 * PLAYER_RADIUS,
                            height: 1.5 * PLAYER_RADIUS,
                        },
                        sight: Sight(Vec2::new(1., 0.)),
                        speed: Speed::default(),
                        room: Room(current_room),
                    },
                    reload: Reload::default(),
                    health: Health::Full,
                    item: Item::Sword,
                };
                commands.spawn(player);
                for room in result_rooms {
                    commands.spawn_batch(room.1);
                    commands.spawn_batch(room.2);
                }
                rooms
                    .iter()
                    .flat_map(|room| room.doors.iter().map(|door| (room.id, door)))
                    .for_each(|(from, door)| {
                        commands.spawn(Door::new(
                            Room(from),
                            Room(door.to),
                            door.direction,
                            door.closed,
                        ));
                    });
                commands.spawn((
                    Door::new(Room(current_room), Room(u8::MAX), enter, false),
                    Entrance,
                ));
            }
        }
    }
}

pub fn respawn_on_death(player: Query<&Health, With<Player>>, mut commands: Commands) {
    if !(player
        .get_single()
        .map(|health| health == &Health::Dead)
        .unwrap_or_default())
    {
        return;
    };
    if is_key_pressed(KeyCode::R) {
        commands.insert_resource(StateChange::Restart);
    }
}

pub fn death_screen(player: Query<&Health, With<Player>>, screen: Res<Screen>) {
    if !(player
        .get_single()
        .map(|health| health == &Health::Dead)
        .unwrap_or_default())
    {
        return;
    };
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

#[derive(Clone, serde::Deserialize, Component, PartialEq, Eq)]
pub enum Item {
    Sword,
    Key,
    Vegetable { name: String, idx: usize },
}

#[derive(Component)]
pub struct Crate;

#[derive(Bundle)]
pub struct ItemCrate {
    item_crate: Crate,
    item: Item,
    position: Position,
    form: Form,
    room: Room,
}

impl ItemCrate {
    pub fn new(item: Item, position: Position, room: Room) -> Self {
        Self {
            item_crate: Crate,
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

pub fn swap_items(
    mut player: Query<(&Position, &Form, &Room, &mut Item), (With<Player>, Without<Crate>)>,
    mut crates: Query<(&Position, &Form, &Room, &mut Item), (With<Crate>, Without<Player>)>,
) {
    let Ok((player_position, player_form, player_room, mut player_item)) = player.get_single_mut() else {
        return;
    };
    for (position, form, room, mut item) in crates.iter_mut() {
        if room.0 != player_room.0 {
            continue;
        }
        let diff = position.0 - player_position.0;
        if is_key_pressed(KeyCode::E)
            && diff.length() <= player_form.direction_len(diff) + form.direction_len(diff) + 0.02
        {
            (*player_item, *item) = (item.clone(), player_item.clone());
        }
    }
}
