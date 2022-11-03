use macroquad::prelude::*;

const RATIO_W_H: f32 = 16. / 9.;

const PLAYER_SPEED: f32 = 0.01;
const BALL_SPEED: f32 = 0.01;
const PLAYER_RADIUS: f32 = 0.025;
const BALL_RADIUS: f32 = 0.005;
const WALL_SIZE: f32 = 0.02;
const PLAYER_RELOAD: f32 = 0.5;

struct Screen {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct Player {
    position: Vec2,
    hiding: bool,
    reload: f32,
}

struct Ball {
    position: Vec2,
    direction: Vec2,
    from_player: bool,
}

struct State {
    player: Player,
    balls: Vec<Ball>,
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
    let mut state = State {
        player: Player {
            position: Vec2 { x: 0.1, y: 0.5 },
            hiding: false,
            reload: 0.,
        },
        balls: vec![],
    };
    loop {
        let dt = get_frame_time();
        let screen = get_screen_size(screen_width(), screen_height());

        // Draw screen
        clear_background(BLACK);
        draw_rectangle(screen.x, screen.y, screen.width, screen.height, WHITE);

        change_state(&mut state, &screen, dt);
        draw(&state, screen);

        next_frame().await
    }
}

/// This function changes state using the controls
fn change_state(state: &mut State, screen: &Screen, dt: f32) {
    if is_key_down(KeyCode::W) {
        state.player.position.y -= PLAYER_SPEED;
    }
    if is_key_down(KeyCode::S) {
        state.player.position.y += PLAYER_SPEED;
    }
    if is_key_down(KeyCode::A) {
        state.player.position.x -= PLAYER_SPEED;
    }
    if is_key_down(KeyCode::D) {
        state.player.position.x += PLAYER_SPEED;
    }

    // wall collision
    state.player.position.x = clamp(
        state.player.position.x,
        WALL_SIZE + PLAYER_RADIUS,
        RATIO_W_H - WALL_SIZE - PLAYER_RADIUS,
    );
    state.player.position.y = clamp(
        state.player.position.y,
        WALL_SIZE + PLAYER_RADIUS,
        1. - WALL_SIZE - PLAYER_RADIUS,
    );

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
            position: ball.position + ball.direction * BALL_SPEED,
            direction: ball.direction,
            from_player: ball.from_player,
        })
        .collect();

    if is_key_pressed(KeyCode::Space) {
        state.player.hiding = !state.player.hiding;
    }
    if is_mouse_button_down(MouseButton::Left) && !state.player.hiding && state.player.reload == 0.
    {
        state.player.reload = PLAYER_RELOAD;
        let (x_mouse, y_mouse) = {
            let (x_m, y_m) = mouse_position();
            (
                clamp((x_m - screen.x) / screen.height, 0., RATIO_W_H),
                clamp((y_m - screen.y) / screen.height, 0., 1.),
            )
        };
        let x_direction = x_mouse - state.player.position.x;
        let y_direction = y_mouse - state.player.position.y;

        let direction = Vec2 {
            x: x_direction,
            y: y_direction,
        }
        .normalize();
        let position = state.player.position + (direction * PLAYER_RADIUS);
        state.balls.push(Ball {
            position,
            direction,
            from_player: true,
        });
    } else {
        state.player.reload = clamp(state.player.reload - dt, 0., PLAYER_RELOAD);
    }
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
    )
}

/// This function draws the state to the screen
fn draw(state: &State, screen: Screen) {
    // Walls
    draw_rect(&screen, 0., 0., RATIO_W_H, 1., GRAY);
    draw_rect(
        &screen,
        WALL_SIZE,
        WALL_SIZE,
        RATIO_W_H - 2. * WALL_SIZE,
        1. - 2. * WALL_SIZE,
        WHITE,
    );
    draw_circ(
        &screen,
        state.player.position.x,
        state.player.position.y,
        PLAYER_RADIUS,
        if state.player.hiding { BLUE } else { GREEN },
    );

    let (x_mouse, y_mouse) = {
        let (x_m, y_m) = mouse_position();
        (
            clamp((x_m - screen.x) / screen.height, 0., RATIO_W_H),
            clamp((y_m - screen.y) / screen.height, 0., 1.),
        )
    };
    draw_lin(
        &screen,
        state.player.position.x,
        state.player.position.y,
        x_mouse,
        y_mouse,
        BALL_RADIUS,
        GRAY,
    );
    for ball in &state.balls {
        draw_circ(&screen, ball.position.x, ball.position.y, BALL_RADIUS, RED);
    }
}
