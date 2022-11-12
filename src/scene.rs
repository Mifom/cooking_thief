use std::io::Read;

use macroquad::{
    prelude::{Color, FileError, Vec2, WHITE},
    text::measure_text,
    texture::{draw_texture_ex, load_texture, DrawTextureParams, Texture2D},
};
use serde::Deserialize;

use crate::{
    graphics::{draw_rect, draw_txt, get_lines, Screen},
    util::RATIO_W_H,
};

const LETTERS_PER_SECOND: f32 = 30.0;

enum State {
    Printing(f32),
    View,
}

#[derive(Debug)]
pub enum Error {
    Parse(serde_yaml::Error),
    OpenAsset(FileError),
}

pub struct Scene {
    cards: Vec<Card>,
    current: usize,
    background: Texture2D,
}

struct Card {
    text: String,
    state: State,
    image: Option<Texture2D>,
}

#[derive(Deserialize)]
struct CardConfig {
    text: String,
    image: Option<String>,
}

#[derive(Deserialize)]
struct SceneConfig {
    cards: Vec<CardConfig>,
    background: String,
}

impl Card {
    const fn new(text: String, image: Option<Texture2D>) -> Self {
        Self {
            text,
            image,
            state: State::Printing(0.0),
        }
    }

    fn reset(&mut self) {
        self.state = State::Printing(0.0);
    }

    fn update(&mut self, skip: bool, dt: f32) -> bool {
        match &mut self.state {
            State::Printing(letters) => {
                *letters += dt * LETTERS_PER_SECOND;
                if skip || *letters > self.text.len() as f32 {
                    self.state = State::View;
                }
                false
            }
            State::View => skip,
        }
    }

    fn draw(&self, screen: &Screen) {
        let text = match self.state {
            State::Printing(letters) => &self.text[0..(letters.floor() as usize)],
            State::View => &self.text,
        };
        if let Some(image) = self.image {
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
            screen,
            0.05,
            0.55,
            RATIO_W_H - 0.1,
            0.4,
            Color::from_rgba(0, 0, 0, 128),
        );
        let (lines, _) = get_lines(screen, RATIO_W_H - 0.2, 0.075, text);
        for (n, line) in lines.into_iter().enumerate() {
            draw_txt(screen, line, 0.1, 0.65 + (0.1 * n as f32), 0.075, WHITE);
        }
    }
}

impl Scene {
    pub async fn from_string(str: impl AsRef<str>) -> Result<Self, Error> {
        let config = serde_yaml::from_str(str.as_ref()).map_err(Error::Parse)?;
        Self::from_config(config).await
    }

    pub async fn from_reader<R: Read>(reader: R) -> Result<Self, Error> {
        let config = serde_yaml::from_reader(reader).map_err(Error::Parse)?;
        Self::from_config(config).await
    }

    async fn from_config(config: SceneConfig) -> Result<Self, Error> {
        let background = load_texture(&format!("assets/{}.png", config.background))
            .await
            .map_err(Error::OpenAsset)?;
        let mut cards = Vec::with_capacity(config.cards.len());
        for card in config.cards {
            let image = if let Some(image) = card.image {
                Some(
                    load_texture(&format!("assets/{image}.png"))
                        .await
                        .map_err(Error::OpenAsset)?,
                )
            } else {
                None
            };
            cards.push(Card::new(card.text, image));
        }
        Ok(Self {
            cards,
            background,
            current: 0,
        })
    }

    pub fn update(&mut self, move_forward: Option<bool>, dt: f32) -> bool {
        if move_forward == Some(false) {
            self.current = self.current.saturating_sub(1);
            false
        } else {
            let skip = move_forward.is_some();
            if self.cards[self.current].update(skip, dt) {
                self.current += 1;
                self.cards.get_mut(self.current).map(Card::reset);
            }
            self.current >= self.cards.len()
        }
    }

    pub fn draw(&self, screen: &Screen) {
        draw_texture_ex(
            self.background,
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
        self.cards[self.current].draw(screen);
    }
}
