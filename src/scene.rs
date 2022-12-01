use macroquad::{
    prelude::{is_key_pressed, is_mouse_button_pressed, Color, KeyCode, MouseButton, Vec2, WHITE},
    texture::{draw_texture_ex, DrawTextureParams},
};
use serde::Deserialize;

use crate::{
    assets::Assets,
    graphics::{draw_rect, draw_txt, get_lines, Screen},
    RATIO_W_H,
};

pub const LETTERS_PER_SECOND: f32 = 30.0;

#[derive(Clone)]
pub enum State {
    Printing(f32),
    View,
}
impl Default for State {
    fn default() -> Self {
        Self::Printing(0.)
    }
}

#[derive(Deserialize, Clone)]
pub struct Scene {
    pub cards: Vec<Card>,
    #[serde(skip)]
    pub current: usize,
    pub background: String,
}

#[derive(Deserialize, Clone)]
pub struct Card {
    pub text: String,
    #[serde(skip)]
    pub state: State,
    pub image: Option<String>,
}

impl Card {
    pub fn reset(&mut self) {
        self.state = State::Printing(0.0);
    }
    pub fn skip(&mut self) -> bool {
        match self.state {
            State::Printing(_) => {
                self.state = State::View;
                false
            }
            State::View => true,
        }
    }
}

pub fn update_scene(scene: &mut Scene, dt: f32) -> bool {
    let current = scene.current;
    let card = scene.cards.get_mut(current).unwrap();
    if let crate::scene::State::Printing(letters) = &mut card.state {
        *letters += dt * LETTERS_PER_SECOND;
        if *letters > card.text.len() as f32 {
            card.state = crate::scene::State::View;
        }
    }
    let forward = is_key_pressed(KeyCode::Space)
        || is_key_pressed(KeyCode::Enter)
        || is_key_pressed(KeyCode::D)
        || is_key_pressed(KeyCode::Right)
        || is_mouse_button_pressed(MouseButton::Left);
    if forward && card.skip() {
        scene.current += 1;

        scene.cards.get_mut(current + 1).map(Card::reset);

        if scene.current >= scene.cards.len() {
            scene.current -= 1;
            return true;
        }
    }
    if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
        scene.current = scene.current.saturating_sub(1);
    }
    false
}

pub fn draw_scene(scene: &Scene, assets: &Assets, screen: &Screen) {
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
        crate::scene::State::Printing(letters) => &card.text[0..(letters.floor() as usize)],
        crate::scene::State::View => &card.text,
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
