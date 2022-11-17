use bevy_ecs::system::{Commands, Res, ResMut, Resource};
use macroquad::{
    prelude::{is_key_pressed, is_mouse_button_pressed, Color, KeyCode, MouseButton, Vec2, WHITE},
    texture::{draw_texture_ex, DrawTextureParams},
};
use serde::Deserialize;

use crate::{
    assets::Assets,
    graphics::{draw_rect, draw_txt, get_lines, Screen},
    util::{StateChange, Time, RATIO_W_H},
};

const LETTERS_PER_SECOND: f32 = 30.0;

#[derive(Clone)]
enum State {
    Printing(f32),
    View,
}
impl Default for State {
    fn default() -> Self {
        Self::Printing(0.)
    }
}

#[derive(Resource, Deserialize, Clone)]
pub struct Scene {
    cards: Vec<Card>,
    #[serde(skip)]
    current: usize,
    background: String,
}

#[derive(Deserialize, Clone)]
pub struct Card {
    text: String,
    #[serde(skip)]
    state: State,
    image: Option<String>,
}

impl Card {
    fn reset(&mut self) {
        self.state = State::Printing(0.0);
    }
    fn skip(&mut self) -> bool {
        match self.state {
            State::Printing(_) => {
                self.state = State::View;
                false
            }
            State::View => true,
        }
    }
}

pub fn update_scene(mut commands: Commands, scene: Option<ResMut<Scene>>, time: Res<Time>) {
    if let Some(mut scene) = scene {
        let current = scene.current;
        let card = scene.cards.get_mut(current).unwrap();
        if let State::Printing(letters) = &mut card.state {
            *letters += time.dt * LETTERS_PER_SECOND;
            if *letters > card.text.len() as f32 {
                card.state = State::View;
            }
        }
        let forward = is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::D)
            || is_key_pressed(KeyCode::Right)
            || is_mouse_button_pressed(MouseButton::Left);
        if forward && card.skip() {
            scene.current += 1;

            scene.cards.get_mut(current + 1).map(Card::reset);

            if scene.current >= scene.cards.len() {
                scene.current -= 1;
                commands.insert_resource(StateChange::Next);
            }
        }
        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            scene.current = scene.current.saturating_sub(1);
        }
    }
}

pub fn draw_scene(scene: Option<Res<Scene>>, screen: Res<Screen>, assets: Res<Assets>) {
    if let Some(scene) = scene {
        draw_texture_ex(
            assets.images[&scene.background],
            screen.x,
            screen.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: screen.width,
                    y: screen.height,
                }),
                ..Default::default()
            },
        );
        let card = &scene.cards[scene.current];
        let text = match card.state {
            State::Printing(letters) => &card.text[0..(letters.floor() as usize)],
            State::View => &card.text,
        };
        if let Some(image) = &card.image {
            let image = assets.images[image];
            let coef = screen.height / image.height();
            draw_texture_ex(
                image,
                screen.x + (screen.width - image.width() * coef) / 2.,
                screen.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2 {
                        x: image.width() * coef,
                        y: screen.height,
                    }),
                    ..Default::default()
                },
            );
        }
        draw_rect(
            &screen,
            0.05,
            0.55,
            RATIO_W_H - 0.1,
            0.4,
            Color::from_rgba(0, 0, 0, 128),
        );
        let (lines, _) = get_lines(&screen, RATIO_W_H - 0.2, 0.075, text);
        for (n, line) in lines.into_iter().enumerate() {
            draw_txt(&screen, line, 0.1, 0.65 + (0.1 * n as f32), 0.075, WHITE);
        }
    }
}
