#![warn(clippy::semicolon_if_nothing_returned)]
use graphics::{draw_centered_txt, draw_rect, get_screen_size, Screen};
use level::Level;
use scene::Scene;
use std::{fs::File, io::BufReader, process::exit};
use util::*;

use macroquad::prelude::*;

mod ai;
mod graphics;
mod level;
mod scene;
mod util;

enum State {
    Scene(Scene),
    Battle(Level, Option<bool>),
}

#[macroquad::main("The Truthy Scroll")]
async fn main() {
    let file = File::open("assets/scene_1.yaml").unwrap();
    let mut state = State::Scene(
        Scene::from_reader(BufReader::new(file))
            .await
            .expect("TODO"),
    );

    loop {
        let dt = get_frame_time();
        let screen = get_screen_size(screen_width(), screen_height());

        // Draw screen
        clear_background(BLACK);
        draw_rectangle(screen.x, screen.y, screen.width, screen.height, WHITE);

        change_state(&mut state, &screen, dt).await;
        draw(&state, &screen);

        next_frame().await;
    }
}

async fn change_state(state: &mut State, screen: &Screen, dt: f32) {
    match state {
        State::Scene(scene) => {
            let forward = is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::D)
                || is_key_pressed(KeyCode::Right)
                || is_mouse_button_pressed(MouseButton::Left);
            let backward = is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left);
            let move_forward = match (forward, backward) {
                (true, false) => Some(true),
                (false, true) => Some(false),
                _ => None,
            };
            let next = scene.update(move_forward, dt);
            if next {
                let file = File::open("assets/level_1.yaml").unwrap();
                *state = State::Battle(
                    Level::from_reader(BufReader::new(file))
                        .await
                        .expect("TODO"),
                    None,
                );
            }
        }
        State::Battle(level, result) => {
            if let Some(win) = result {
                if is_key_pressed(KeyCode::Q) {
                    exit(0)
                } else if is_key_pressed(KeyCode::R) {
                    if *win {
                        *state = State::Scene(
                            Scene::from_reader(BufReader::new(
                                File::open("assets/scene_1.yaml").unwrap(),
                            ))
                            .await
                            .expect("TODO"),
                        );
                    } else {
                        level.restart().await;
                        *result = None;
                    }
                }
            } else {
                *result = change_battle_state(level, screen, dt);
            }
        }
    }
}

/// This function changes state of battle using the controls
/// Returns Some(win) if battle is over
fn change_battle_state(map: &mut Level, screen: &Screen, dt: f32) -> Option<bool> {
    let mut move_direction = (0, 0);
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        move_direction.1 -= 1;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        move_direction.1 += 1;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        move_direction.0 -= 1;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
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
        State::Scene(scene) => scene.draw(screen),
        State::Battle(level, going) => {
            level.draw(screen);
            if let Some(win) = going {
                let color = if *win {
                    BLACK
                } else {
                    Color::from_rgba(128, 0, 0, 128)
                };
                draw_rect(screen, 0., 0., RATIO_W_H, 1., color);
                draw_centered_txt(
                    screen,
                    &format!(
                        "You {}, press R to restart",
                        if *win { "win" } else { "lose" }
                    ),
                    0.5,
                    0.1,
                    WHITE,
                );
            }
        }
    }
    #[cfg(debug_assertions)]
    draw_text(&format!("{}", get_fps()), 10., 40., 30., YELLOW);
}
