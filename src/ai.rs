use macroquad::prelude::Vec2;

use crate::util::{Body, MoveAction, PLAYER_RADIUS, SLASH_LEN};

enum State {
    Idle,
    Fight(Vec2),
    LastSeen(Vec2, f32),
}

pub struct BasicAi {
    state: State,
    position: Vec2,
}

impl BasicAi {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            state: State::Idle,
        }
    }
}

impl BasicAi {
    pub fn action(&mut self, enemy: &Body, player: Option<Vec2>, dt: f32) -> (MoveAction, bool) {
        self.state = if let Some(position) = player {
            State::Fight(position)
        } else {
            match self.state {
                State::Fight(position) => State::LastSeen(position, dt),
                State::Idle => State::Idle,
                State::LastSeen(position, time) => {
                    let new_time = time + dt;
                    if new_time > 5. {
                        State::Idle
                    } else {
                        State::LastSeen(position, new_time)
                    }
                }
            }
        };
        match self.state {
            State::Idle => (
                MoveAction {
                    move_direction: enemy.move_to(self.position),
                    sight: Vec2 { x: 1., y: 0. },
                },
                false,
            ),
            State::Fight(position) => (
                MoveAction {
                    move_direction: enemy.move_to(position),
                    sight: (position - enemy.position).normalize(),
                },
                enemy.position.distance(position) < 2. * PLAYER_RADIUS + SLASH_LEN,
            ),
            State::LastSeen(position, _) => (
                MoveAction {
                    move_direction: enemy.move_to(position),
                    sight: Vec2 { x: 1., y: 0. },
                },
                false,
            ),
        }
    }
}
