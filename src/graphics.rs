#![allow(unused)]
use bevy_ecs::system::{Res, Resource};
use macroquad::{
    prelude::{Color, BLACK, WHITE},
    shapes::{draw_circle, draw_line, draw_rectangle},
    text::{draw_text, measure_text},
    window::clear_background,
};

use crate::util::RATIO_W_H;

#[derive(Resource)]
pub struct Screen {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Gets screen size from window size for the defined ratio
pub fn get_screen_size(width: f32, height: f32) -> Screen {
    if width / height > RATIO_W_H {
        let new_width = height * RATIO_W_H;
        Screen {
            x: (width - new_width) / 2.,
            y: 0.,
            width: new_width,
            height,
        }
    } else {
        let new_height = width / RATIO_W_H;
        Screen {
            x: 0.,
            y: (height - new_height) / 2.,
            width,
            height: new_height,
        }
    }
}

pub fn draw_screen(screen: Res<Screen>) {
    clear_background(BLACK);
    draw_rectangle(screen.x, screen.y, screen.width, screen.height, WHITE);
}

pub fn draw_rect(screen: &Screen, x: f32, y: f32, w: f32, h: f32, color: Color) {
    debug_assert!((0. ..=RATIO_W_H).contains(&x));
    debug_assert!((0. ..=1.).contains(&y));
    debug_assert!((0. ..=RATIO_W_H).contains(&w));
    debug_assert!((0. ..=1.).contains(&h));
    draw_rectangle(
        screen.height * x + screen.x,
        screen.height * y + screen.y,
        screen.height * w,
        screen.height * h,
        color,
    );
}

pub fn draw_circ(screen: &Screen, x: f32, y: f32, r: f32, color: Color) {
    debug_assert!((0. ..=RATIO_W_H).contains(&x));
    debug_assert!((0. ..=1.).contains(&y));
    debug_assert!((0. ..=1.).contains(&r));
    draw_circle(
        screen.height * x + screen.x,
        screen.height * y + screen.y,
        screen.height * r,
        color,
    );
}

pub fn get_lines<'a>(
    screen: &Screen,
    max_text_width: f32,
    text_size: f32,
    text: &'a str,
) -> (Vec<&'a str>, f32) {
    let mut result = vec![&text[0..0]];
    let mut whitespaces: Vec<_> = text
        .char_indices()
        .filter_map(|(n, ch)| (ch.is_whitespace()).then_some(n))
        .collect();
    whitespaces.push(text.len());
    let mut start = 0;
    let mut end = 0;
    let mut max_len = 0.;
    for whitespace in whitespaces {
        let dims = measure_text(
            &text[start..whitespace],
            None,
            (text_size * screen.height) as u16,
            1.0,
        );
        if dims.width > max_text_width * screen.height {
            start = end + 1;
            result.push(&text[start..whitespace]);
        } else {
            end = whitespace;
            if max_len < dims.width {
                max_len = dims.width;
            }
            if let Some(last) = result.last_mut() {
                *last = &text[start..end];
            }
        }
    }
    (result, max_len / screen.height)
}

pub fn draw_lin(screen: &Screen, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color) {
    debug_assert!((0. ..=RATIO_W_H).contains(&x1));
    debug_assert!((0. ..=1.).contains(&y1));
    debug_assert!((0. ..=RATIO_W_H).contains(&x2));
    debug_assert!((0. ..=1.).contains(&y2));
    debug_assert!((0. ..=RATIO_W_H).contains(&width));
    draw_line(
        x1 * screen.height + screen.x,
        y1 * screen.height + screen.y,
        x2 * screen.height + screen.x,
        y2 * screen.height + screen.y,
        width * screen.height,
        color,
    );
}

pub fn draw_txt(screen: &Screen, text: &str, x: f32, y: f32, font: f32, color: Color) {
    debug_assert!((0. ..=RATIO_W_H).contains(&x));
    debug_assert!((0. ..=1.).contains(&y));
    debug_assert!((0. ..=1.).contains(&font));
    draw_text(
        text,
        screen.height * x + screen.x,
        screen.height * y + screen.y,
        screen.height * font,
        color,
    );
}

pub fn draw_centered_txt(screen: &Screen, text: &str, y: f32, font: f32, color: Color) {
    debug_assert!((0. ..=1.).contains(&y));
    debug_assert!((0. ..=1.).contains(&font));
    let text_dims = measure_text(text, None, (screen.height * font) as u16, 1.);
    let x = (RATIO_W_H - text_dims.width / screen.height) / 2.;
    draw_text(
        text,
        screen.height * x + screen.x,
        screen.height * y + screen.y,
        screen.height * font,
        color,
    );
}
