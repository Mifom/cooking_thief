#![warn(clippy::semicolon_if_nothing_returned)]
use graphics::{draw_centered_text, get_screen_size, Screen};
use level::Level;
use std::{fs::File, io::BufReader, process::exit};
use util::*;

use macroquad::prelude::*;

mod ai;
mod graphics;
mod level;
mod util;

enum State {
    Battle(Level),
    Restart(bool),
}

#[macroquad::main("The Truthy Scroll")]
async fn main() {
    let file = File::open("assets/test.yaml").unwrap();
    let mut state = State::Battle(Level::from_reader(BufReader::new(file)).expect("TODO"));
    loop {
        let dt = get_frame_time();
        let screen = get_screen_size(screen_width(), screen_height());

        // Draw screen
        clear_background(BLACK);
        draw_rectangle(screen.x, screen.y, screen.width, screen.height, WHITE);

        change_state(&mut state, &screen, dt);
        draw(&state, &screen);

        next_frame().await;
    }
}

fn change_state(state: &mut State, screen: &Screen, dt: f32) {
    match state {
        State::Battle(battle_state) => {
            if let Some(win) = change_battle_state(battle_state, screen, dt) {
                *state = State::Restart(win);
            }
        }
        State::Restart(_) => {
            if is_key_pressed(KeyCode::Q) {
                exit(0)
            } else if is_key_pressed(KeyCode::R) {
                *state = State::Battle(
                    Level::from_reader(BufReader::new(File::open("assets/test.yaml").unwrap()))
                        .unwrap(),
                );
            }
        }
    }
}

/// This function changes state of battle using the controls
/// Returns Some(win) if battle is over
fn change_battle_state(map: &mut Level, screen: &Screen, dt: f32) -> Option<bool> {
    let mut move_direction = (0, 0);
    if is_key_down(KeyCode::W) {
        move_direction.1 -= 1;
    }
    if is_key_down(KeyCode::S) {
        move_direction.1 += 1;
    }
    if is_key_down(KeyCode::A) {
        move_direction.0 -= 1;
    }
    if is_key_down(KeyCode::D) {
        move_direction.0 += 1;
    }
    let (x_mouse, y_mouse) = {
        let (x_m, y_m) = mouse_position();
        (
            clamp((x_m - screen.x) / screen.height, 0., RATIO_W_H),
            clamp((y_m - screen.y) / screen.height, 0., 1.),
        )
    };

    let player_action = PlayerAction {
        move_direction,
        view_point: Vec2 {
            x: x_mouse,
            y: y_mouse,
        },
        toggle_visibility: is_key_pressed(KeyCode::Space),
        shoot: is_mouse_button_down(MouseButton::Left),
    };

    map.update(player_action, dt)
}

/// This function draws the state to the screen
fn draw(state: &State, screen: &Screen) {
    match state {
        State::Battle(map) => map.draw(screen),
        State::Restart(win) => draw_centered_text(
            screen,
            &format!(
                "You {}, press R to restart",
                if *win { "win" } else { "lose" }
            ),
            0.5,
            0.1,
            BLACK,
        ),
    }
    #[cfg(debug_assertions)]
    draw_text(&format!("{}", get_fps()), 10., 40., 30., YELLOW);
}
