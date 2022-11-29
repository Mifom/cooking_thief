#![warn(clippy::semicolon_if_nothing_returned)]
use assets::SCENES;
use graphics::{draw_centered_txt, draw_cursor, draw_rect, get_screen_size, Screen};
use level::{draw_level, update_level, Level};
use scene::{draw_scene, update_scene, Scene};

use macroquad::{
    audio::{play_sound, stop_sound, PlaySoundParams},
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
    End,
}

#[macroquad::main("Cooking thief")]
async fn main() {
    show_mouse(false);

    let assets = Assets::load().await;
    let mut state = State::Scene(0, assets.scenes[0].clone());

    loop {
        let dt = get_frame_time();
        let screen = get_screen_size(screen_width(), screen_height());

        update(&mut state, &screen, &assets, dt);

        draw(&screen, &state, &assets);
        // world.insert_resource(screen);
        // world.get_resource_mut::<Time>().unwrap().update(dt);

        // schedule.run(&mut world);

        next_frame().await;
    }
}
pub fn update(state: &mut crate::State, screen: &Screen, assets: &Assets, dt: f32) {
    let next = match state {
        crate::State::Scene(_, scene) => update_scene(scene, dt),
        crate::State::Battle(_, level) => update_level(level, screen, assets, dt),
        crate::State::End => is_key_pressed(KeyCode::Q),
    };
    if next {
        change_state(state, assets);
    }
}

fn change_state(state: &mut crate::State, assets: &Assets) {
    let sound = assets.sounds.get("stealth").unwrap();
    stop_sound(sound.clone());
    *state = match state {
        crate::State::Scene(num, _) => {
            let config = assets.levels.get(*num).unwrap();
            play_sound(
                sound.clone(),
                PlaySoundParams {
                    looped: true,
                    volume: 1.,
                },
            );

            crate::State::Battle(*num, Level::load(config))
        }
        crate::State::Battle(num, _) => {
            let new_num = *num + 1;
            if new_num < SCENES.len() {
                crate::State::Scene(new_num, assets.scenes[new_num].clone())
            } else {
                crate::State::End
            }
        }
        crate::State::End => std::process::exit(0),
    };
}

pub fn draw(screen: &Screen, state: &crate::State, assets: &Assets) {
    clear_background(BLACK);
    draw_rectangle(screen.x, screen.y, screen.width, screen.height, WHITE);
    match state {
        crate::State::Scene(_, scene) => draw_scene(scene, assets, screen),
        crate::State::Battle(_, level) => draw_level(level, assets, screen),
        crate::State::End => {
            draw_rect(screen, 0., 0., RATIO_W_H, 1., BLACK);
            draw_centered_txt(screen, "That was hard. Press Q to quit.", 0.4, 0.08, WHITE);
        }
    }

    draw_cursor(state, assets, screen);
}
