use macroquad::prelude::Vec2;

use crate::util::{Body, Form, MoveAction, Phrase, SLASH_LEN};

enum State {
    Idle,
    Fight(Vec2, Form),
    LastSeen(Vec2, f32),
}

pub struct BasicAi {
    state: State,
    position: Vec2,
}

impl BasicAi {
    pub const fn new(position: Vec2) -> Self {
        Self {
            position,
            state: State::Idle,
        }
    }
}

impl BasicAi {
    pub fn action(
        &mut self,
        enemy: &Body,
        player: Option<&Body>,
        dt: f32,
    ) -> (MoveAction, bool, Option<Phrase>) {
        let mut phrase = None;
        self.state = if let Some(body) = player {
            if !matches!(self.state, State::Fight(_, _)) {
                phrase = Some(Phrase {
                    text: "Here you are!".to_owned(),
                    time: 1.,
                });
            }
            State::Fight(body.position, body.form)
        } else {
            match self.state {
                State::Fight(position, _) => {
                    phrase = Some(Phrase {
                        text: "Where is he?".to_owned(),
                        time: 2.,
                    });
                    State::LastSeen(position, dt)
                }
                State::Idle => State::Idle,
                State::LastSeen(position, time) => {
                    let new_time = time + dt;
                    if new_time > 5. {
                        phrase = Some(Phrase {
                            text: "Must've been wind".to_owned(),
                            time: 2.,
                        });
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
                phrase,
            ),
            State::Fight(position, form) => {
                let diff = position - enemy.position;
                (
                    MoveAction {
                        move_direction: enemy.move_to(position),
                        sight: (position - enemy.position).normalize(),
                    },
                    diff.length()
                        < enemy.form.direction_len(diff) + form.direction_len(diff) + SLASH_LEN,
                    phrase,
                )
            }
            State::LastSeen(position, _) => (
                MoveAction {
                    move_direction: enemy.move_to(position),
                    sight: Vec2 { x: 1., y: 0. },
                },
                false,
                phrase,
            ),
        }
    }
}
