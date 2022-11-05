#![warn(clippy::semicolon_if_nothing_returned)]
use std::process::exit;

use ::rand::{thread_rng, Rng};
use macroquad::prelude::*;

const RATIO_W_H: f32 = 16. / 9.;

const BALL_SPEED: f32 = 1.75;
const PLAYER_RADIUS: f32 = 0.025;
const BALL_RADIUS: f32 = 0.01;
const WALL_SIZE: f32 = 0.02;

const SPEED_STEPS: i32 = 10;
const PLAYER_MAX_SPEED: f32 = 0.65;
const PLAYER_RELOAD: f32 = 0.5;
const DASH_LEN: f32 = 0.02;

struct Screen {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct Speed {
    x: i32,
    y: i32,
}

struct Body {
    position: Vec2,
    sight: Vec2,
    speed: Speed,
}

impl Body {
    fn update(&mut self, move_direction: (i32, i32), sight: Vec2, dt: f32) {
        self.sight = sight.normalize();
        self.speed.x += 2 * move_direction.0;
        self.speed.y += 2 * move_direction.1;

        match self.speed.x.cmp(&0) {
            std::cmp::Ordering::Less => self.speed.x += 1,
            std::cmp::Ordering::Greater => self.speed.x -= 1,
            _ => {}
        }
        self.speed.x = clamp(self.speed.x, -SPEED_STEPS, SPEED_STEPS);
        match self.speed.y.cmp(&0) {
            std::cmp::Ordering::Less => self.speed.y += 1,
            std::cmp::Ordering::Greater => self.speed.y -= 1,
            _ => {}
        }
        self.speed.y = clamp(self.speed.y, -SPEED_STEPS, SPEED_STEPS);
        self.position.x += PLAYER_MAX_SPEED * (self.speed.x as f32) / (SPEED_STEPS as f32) * dt;
        self.position.y += PLAYER_MAX_SPEED * (self.speed.y as f32) / (SPEED_STEPS as f32) * dt;

        // wall collision
        self.position.x = clamp(
            self.position.x,
            WALL_SIZE + PLAYER_RADIUS,
            RATIO_W_H - WALL_SIZE - PLAYER_RADIUS,
        );
        self.position.y = clamp(
            self.position.y,
            WALL_SIZE + PLAYER_RADIUS,
            1. - WALL_SIZE - PLAYER_RADIUS,
        );
    }

    fn collide(&mut self, other: &mut Self) {
        let diff = self.position - other.position;
        let penetration = PLAYER_RADIUS - (diff.length() / 2.);
        if penetration > 0. {
            let shift = diff.normalize() * penetration;
            self.position += shift;
            other.position -= shift;
        }
    }
}

struct Player {
    body: Body,
    visible: bool,
    reload: f32,
    low_health: bool,
}

struct Ball {
    position: Vec2,
    direction: Vec2,
}

struct Enemy {
    body: Body,
    reload: f32,
    slash: i8,
}

struct BattleState {
    player: Player,
    balls: Vec<Ball>,
    enemies: Vec<Enemy>,
}

enum State {
    Battle(BattleState),
    Restart(bool),
}

impl BattleState {
    fn generate(rng: &mut impl Rng) -> Self {
        Self {
            player: Player {
                body: Body {
                    position: Vec2 {
                        x: 0.1,
                        y: rng.gen_range(0.25..=0.75),
                    },
                    speed: Speed { x: 0, y: 0 },
                    sight: Vec2 { x: 1., y: 0. },
                },
                visible: false,
                reload: 0.,
                low_health: false,
            },
            balls: vec![],
            enemies: (1..=rng.gen_range(1..=3))
                .map(|_| Enemy {
                    body: Body {
                        position: Vec2 {
                            x: rng.gen_range(RATIO_W_H / 3.0..2. * RATIO_W_H / 3.),
                            y: rng.gen_range(0.25..=0.75),
                        },
                        speed: Speed { x: 0, y: 0 },
                        sight: Vec2 { x: 1., y: 0. },
                    },
                    reload: 0.,
                    slash: 0,
                })
                .collect(),
        }
    }
}

/// Gets screen size from window size for the defined ratio
fn get_screen_size(width: f32, height: f32) -> Screen {
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

#[macroquad::main("Game")]
async fn main() {
    let mut state = State::Battle(BattleState::generate(&mut thread_rng()));
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
                *state = State::Battle(BattleState::generate(&mut thread_rng()));
            }
        }
    }
}

/// This function changes state of battle using the controls
/// Returns Some(win) if battle is over
fn change_battle_state(state: &mut BattleState, screen: &Screen, dt: f32) -> Option<bool> {
    if state.enemies.is_empty() {
        return Some(true);
    }
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
    let x_direction = x_mouse - state.player.body.position.x;
    let y_direction = y_mouse - state.player.body.position.y;

    let direction = Vec2 {
        x: x_direction,
        y: y_direction,
    }
    .normalize_or_zero();

    state.player.body.update(move_direction, direction, dt);

    state.balls = state
        .balls
        .iter()
        .filter(|ball| {
            ball.position.x >= WALL_SIZE + PLAYER_RADIUS
                && ball.position.x <= RATIO_W_H - WALL_SIZE - PLAYER_RADIUS
                && ball.position.y >= WALL_SIZE + PLAYER_RADIUS
                && ball.position.y <= 1. - WALL_SIZE - PLAYER_RADIUS
        })
        .map(|ball| Ball {
            position: ball.position + ball.direction * BALL_SPEED * dt,
            direction: ball.direction,
        })
        .collect();

    let mut enemy_collisions = Vec::new();
    for enemy in &mut state.enemies {
        let mut move_direction = (0, 0);
        if state.player.visible {
            if enemy.body.position.y > state.player.body.position.y {
                move_direction.1 -= 1;
            } else if enemy.body.position.y < state.player.body.position.y {
                move_direction.1 += 1;
            }
            if enemy.body.position.x > state.player.body.position.x {
                move_direction.0 -= 1;
            } else if enemy.body.position.x < state.player.body.position.x {
                move_direction.0 += 1;
            }
        }
        enemy.body.update(
            move_direction,
            state.player.body.position - enemy.body.position,
            dt,
        );
        enemy.body.collide(&mut state.player.body);
        let dash_feel = if state.player.visible { 1. } else { 0.5 };
        if enemy.body.position.distance(state.player.body.position)
            < 2. * PLAYER_RADIUS + DASH_LEN * dash_feel
            && enemy.reload == 0.
        {
            enemy.reload = PLAYER_RELOAD;
            enemy.slash = 5;
            if state.player.low_health {
                return Some(false);
            } else {
                state.player.low_health = true;
            }
        } else {
            enemy.slash = clamp(enemy.slash - 1, 0, 5);
            enemy.reload = clamp(enemy.reload - dt, 0., PLAYER_RELOAD);
        }
        let mut ball_collisions = Vec::new();
        for ball in &state.balls {
            if enemy.body.position.distance(ball.position) < BALL_RADIUS + PLAYER_RADIUS {
                enemy_collisions.push(enemy.body.position);
                ball_collisions.push(ball.position);
            }
        }
        state
            .balls
            .retain(|ball| !ball_collisions.contains(&ball.position));
    }
    state
        .enemies
        .retain(|enemy| !enemy_collisions.contains(&enemy.body.position));

    if is_key_pressed(KeyCode::Space) {
        state.player.visible = !state.player.visible;
    }
    if is_mouse_button_down(MouseButton::Left) && state.player.visible && state.player.reload == 0.
    {
        state.player.reload = PLAYER_RELOAD;
        let position = state.player.body.position + (state.player.body.sight * PLAYER_RADIUS);
        state.balls.push(Ball {
            position,
            direction,
        });
    } else {
        state.player.reload = clamp(state.player.reload - dt, 0., PLAYER_RELOAD);
    }
    None
}

fn draw_rect(screen: &Screen, x: f32, y: f32, w: f32, h: f32, color: Color) {
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

fn draw_circ(screen: &Screen, x: f32, y: f32, r: f32, color: Color) {
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

fn draw_body(screen: &Screen, body: &Body, color: Color) {
    draw_circ(
        screen,
        body.position.x,
        body.position.y,
        PLAYER_RADIUS,
        color,
    );
}

fn draw_lin(screen: &Screen, x1: f32, y1: f32, x2: f32, y2: f32, width: f32, color: Color) {
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
fn draw_centered_text(screen: &Screen, text: &str, y: f32, font: f32, color: Color) {
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

/// This function draws the state to the screen
fn draw(state: &State, screen: &Screen) {
    match state {
        State::Battle(state) => {
            // Walls
            draw_rect(
                screen,
                0.,
                0.,
                RATIO_W_H,
                1.,
                if state.player.low_health { RED } else { GRAY },
            );
            draw_rect(
                screen,
                WALL_SIZE,
                WALL_SIZE,
                RATIO_W_H - 2. * WALL_SIZE,
                1. - 2. * WALL_SIZE,
                WHITE,
            );

            draw_body(
                screen,
                &state.player.body,
                if state.player.visible { GREEN } else { BLUE },
            );

            // INFO: uncomment if want to see sight trace
            //
            // let (x_mouse, y_mouse) = {
            //     let (x_m, y_m) = mouse_position();
            //     (
            //         clamp((x_m - screen.x) / screen.height, 0., RATIO_W_H),
            //         clamp((y_m - screen.y) / screen.height, 0., 1.),
            //     )
            // };
            // draw_lin(
            //     screen,
            //     state.player.position.x,
            //     state.player.position.y,
            //     x_mouse,
            //     y_mouse,
            //     BALL_RADIUS,
            //     GRAY,
            // );
            for ball in &state.balls {
                draw_circ(screen, ball.position.x, ball.position.y, BALL_RADIUS, RED);
            }
            for enemy in &state.enemies {
                draw_body(screen, &enemy.body, ORANGE);
                if enemy.slash > 0 {
                    let slash_x = enemy.body.sight.x * PLAYER_RADIUS + enemy.body.position.x;
                    let slash_y = enemy.body.sight.y * PLAYER_RADIUS + enemy.body.position.y;
                    let slash_x_end = enemy.body.sight.x * DASH_LEN + slash_x;
                    let slash_y_end = enemy.body.sight.y * DASH_LEN + slash_y;
                    draw_lin(
                        screen,
                        slash_x,
                        slash_y,
                        slash_x_end,
                        slash_y_end,
                        0.05,
                        GRAY,
                    );
                }
            }
        }
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
    draw_text(&format!("{}", get_fps()), 10., 40., 30., YELLOW);
}
