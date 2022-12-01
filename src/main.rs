#![warn(clippy::semicolon_if_nothing_returned)]
use assets::SCENES;
use graphics::{draw_centered_txt, draw_cursor, draw_rect, get_screen_size, Screen};
use level::{draw_level, update_level, Level};
use scene::{draw_scene, update_scene, Scene};

use macroquad::{
    audio::{play_sound, stop_sound, PlaySoundParams, Sound},
    prelude::*,
};

use crate::assets::Assets;

mod assets;
mod graphics;
mod level;
mod scene;

pub const RATIO_W_H: f32 = 16. / 9.;

pub enum State {
    Scene(usize, Scene),
    Battle(usize, Level),
    End(usize),
}

#[macroquad::main("Cooking thief")]
async fn main() {
    show_mouse(false);

    let assets = Assets::load().await;
    // let mut state = State::Scene(0, assets.scenes[0].clone());
    // let mut sound = assets.sounds["village"];
    let mut state = State::End(0);
    let mut sound = assets.sounds["thief_at_the_kitchen"];
    play_sound(
        sound.clone(),
        PlaySoundParams {
            looped: true,
            volume: 0.75,
        },
    );

    loop {
        let dt = get_frame_time();
        let screen = get_screen_size(screen_width(), screen_height());

        update(&mut state, &screen, &assets, &mut sound, dt);

        draw(&screen, &state, &assets);

        next_frame().await;
    }
}
pub fn update(
    state: &mut crate::State,
    screen: &Screen,
    assets: &Assets,
    sound: &mut Sound,
    dt: f32,
) {
    let next = match state {
        crate::State::Scene(_, scene) => update_scene(scene, dt),
        crate::State::Battle(_, level) => update_level(level, screen, assets, dt),
        crate::State::End(pos) => {
            let forward = is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::D)
                || is_key_pressed(KeyCode::Right)
                || is_mouse_button_pressed(MouseButton::Left);
            if forward {
                *pos += 1;
                *pos >= assets.end.len()
            } else {
                false
            }
        }
    };
    if next {
        change_state(state, assets, sound);
    }
}

fn change_state(state: &mut crate::State, assets: &Assets, sound: &mut Sound) {
    stop_sound(sound.clone());
    *state = match state {
        crate::State::Scene(num, _) => {
            let config = assets.levels.get(*num).unwrap();
            *sound = assets.sounds["stealth"];

            crate::State::Battle(*num, Level::load(config))
        }
        crate::State::Battle(num, _) => {
            let new_num = *num + 1;
            if new_num < SCENES.len() {
                *sound = assets.sounds["village"];
                crate::State::Scene(new_num, assets.scenes[new_num].clone())
            } else {
                *sound = assets.sounds["thief_at_the_kitchen"];
                crate::State::End(0)
            }
        }
        crate::State::End(_) => std::process::exit(0),
    };
    play_sound(
        sound.clone(),
        PlaySoundParams {
            looped: true,
            volume: 0.75,
        },
    );
}

pub fn draw(screen: &Screen, state: &crate::State, assets: &Assets) {
    clear_background(BLACK);
    draw_rectangle(screen.x, screen.y, screen.width, screen.height, WHITE);
    match state {
        crate::State::Scene(_, scene) => draw_scene(scene, assets, screen),
        crate::State::Battle(_, level) => draw_level(level, assets, screen),
        crate::State::End(pos) => {
            draw_rect(screen, 0., 0., RATIO_W_H, 1., BLACK);
            let start = 0.5 - 0.04 * assets.end[*pos].len() as f32;
            for (n, line) in assets.end[*pos].iter().enumerate() {
                draw_centered_txt(screen, line, start + 0.08 * (n + 1) as f32, 0.045, WHITE);
            }
        }
    }

    draw_cursor(state, assets, screen);
}
