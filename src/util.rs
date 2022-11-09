use std::cmp::Ordering;

use macroquad::prelude::*;

use crate::ai::BasicAi;

pub const RATIO_W_H: f32 = 16. / 9.;

pub const BALL_SPEED: f32 = 1.75;
pub const PLAYER_RADIUS: f32 = 0.025;
pub const BALL_RADIUS: f32 = 0.01;
pub const WALL_SIZE: f32 = 0.02;

pub const SPEED_STEPS: i32 = 10;
pub const PLAYER_MAX_SPEED: f32 = 0.65;
pub const PLAYER_RELOAD: f32 = 0.5;
pub const SLASH_LEN: f32 = 0.02;

struct Speed {
    x: i32,
    y: i32,
}

pub struct MoveAction {
    pub move_direction: (i32, i32),
    pub sight: Vec2,
}

pub struct PlayerAction {
    pub move_direction: (i32, i32),
    pub view_point: Vec2,
    pub toggle_visibility: bool,
    pub shoot: bool,
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

pub struct Body {
    pub position: Vec2,
    pub form: Form,
    pub sight: Vec2,
    speed: Speed,
}

impl Body {
    pub const fn circle(position: Vec2, radius: f32) -> Self {
        Self {
            position,
            form: Form::Circle { radius },
            sight: Vec2 { x: 1., y: 0. },
            speed: Speed { x: 0, y: 0 },
        }
    }
    pub fn rect(position: Vec2, width: f32, height: f32) -> Self {
        Self {
            position,
            form: Form::Rect {
                width: width / 2.,
                height: height / 2.,
            },
            sight: Vec2 { x: 1., y: 0. },
            speed: Speed { x: 0, y: 0 },
        }
    }

    pub fn update(&mut self, move_action: MoveAction, dt: f32) {
        self.sight = move_action.sight.normalize();
        self.speed.x += 2 * move_action.move_direction.0;
        self.speed.y += 2 * move_action.move_direction.1;

        match self.speed.x.cmp(&0) {
            std::cmp::Ordering::Less => self.speed.x += 1,
            std::cmp::Ordering::Greater => self.speed.x -= 1,
            _ => {}
        }
        self.speed.x = clamp(self.speed.x, -SPEED_STEPS, SPEED_STEPS);
        match self.speed.y.cmp(&0) {
            std::cmp::Ordering::Less => self.speed.y += 1,
            std::cmp::Ordering::Greater => self.speed.y -= 1,
            _ => {}
        }
        self.speed.y = clamp(self.speed.y, -SPEED_STEPS, SPEED_STEPS);
        self.position.x += PLAYER_MAX_SPEED * (self.speed.x as f32) / (SPEED_STEPS as f32) * dt;
        self.position.y += PLAYER_MAX_SPEED * (self.speed.y as f32) / (SPEED_STEPS as f32) * dt;

        // wall collision
        let x_wall = self.form.x_r();
        let y_wall = self.form.y_r();
        self.position.x = clamp(
            self.position.x,
            WALL_SIZE + x_wall,
            RATIO_W_H - WALL_SIZE - x_wall,
        );
        self.position.y = clamp(self.position.y, WALL_SIZE + y_wall, 1. - WALL_SIZE - y_wall);
    }

    pub fn collide(&mut self, other: &mut Self) {
        if let Some(shift) = self.collision(other) {
            self.position += shift;
            other.position -= shift;
        }
    }

    pub fn collision(&self, other: &Self) -> Option<Vec2> {
        let diff = self.position - other.position;
        let size = self.form.direction_len(diff) + other.form.direction_len(diff);
        let penetration = (size - diff.length()) / 2.;
        (penetration > 0.).then(|| diff.normalize() * penetration)
    }

    pub fn move_to(&self, position: Vec2) -> (i32, i32) {
        let mut move_direction = (0, 0);
        if self.position.distance(position) < 1.5 * PLAYER_RADIUS {
            return move_direction;
        }
        if self.position.y > position.y {
            move_direction.1 -= 1;
        } else if self.position.y < position.y {
            move_direction.1 += 1;
        }
        if self.position.x > position.x {
            move_direction.0 -= 1;
        } else if self.position.x < position.x {
            move_direction.0 += 1;
        }
        move_direction
    }
}

pub struct Player {
    pub body: Body,
    pub visible: bool,
    pub reload: f32,
    pub low_health: bool,
    pub model: Texture2D,
}

impl Player {
    pub async fn new(position: Vec2) -> Self {
        Self {
            body: Body::rect(position, 3. * PLAYER_RADIUS, 3. * PLAYER_RADIUS),
            visible: false,
            reload: 0.,
            low_health: false,
            model: load_texture("assets/player.png").await.unwrap(),
        }
    }
}

pub struct Ball {
    pub position: Vec2,
    pub direction: Vec2,
}

pub struct Enemy {
    pub id: u32,
    pub body: Body,
    pub reload: f32,
    pub slash: i8,
    pub actor: BasicAi,
    pub dead: bool,
}

impl Enemy {
    pub const fn new(id: u32, position: Vec2) -> Self {
        Self {
            id,
            body: Body::circle(position, PLAYER_RADIUS),
            reload: 0.,
            slash: 0,
            actor: BasicAi::new(position),
            dead: false,
        }
    }
}

impl PartialEq for Enemy {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Enemy {}
