#![warn(clippy::semicolon_if_nothing_returned)]
use graphics::get_screen_size;
use level::{draw_phrase, draw_player};
use util::*;

use bevy_ecs::prelude::*;
use macroquad::prelude::*;

use crate::{
    assets::Assets,
    graphics::draw_screen,
    level::{draw_balls, draw_doors, draw_enemies},
    scene::{draw_scene, update_scene},
};

mod assets;
mod graphics;
mod level;
mod scene;
mod util;

#[derive(Resource)]
pub enum State {
    Scene(String),
    Battle(String),
}

#[derive(StageLabel)]
pub enum Label {
    First,
    Update,
    Draw,
}

#[macroquad::main("Super cooking simulator")]
async fn main() {
    let mut world = World::new();

    world.insert_resource(Assets::load().await.unwrap());
    world.insert_resource(Time::default());
    world.insert_resource(State::Battle("level_1".to_string()));
    world.insert_resource(Events::<(Entity, MoveAction)>::default());

    let mut schedule = Schedule::default();

    schedule.add_stage(
        Label::First,
        SystemStage::parallel().with_system(Events::<(Entity, MoveAction)>::update_system),
    );

    schedule.add_stage(
        Label::Update,
        SystemStage::parallel()
            .with_system(update_scene)
            .with_system(change_state)
            .with_system(load_new_state)
            .with_system(player_action)
            .with_system(move_body)
            .with_system(collide)
            .with_system(use_doors)
            .with_system(enemies_actions)
            .with_system(update_reload)
            .with_system(update_balls)
            .with_system(collide_balls)
            .with_system(update_phrase)
            .with_system(respawn_on_death),
    );

    schedule.add_stage(
        Label::Draw,
        SystemStage::single_threaded()
            .with_system(draw_screen)
            .with_system(draw_scene.after(draw_screen))
            .with_system(draw_doors.after(draw_screen))
            .with_system(draw_player.after(draw_doors))
            .with_system(draw_balls.after(draw_doors))
            .with_system(draw_enemies.after(draw_doors))
            .with_system(draw_phrase.after(draw_doors))
            .with_system(death_screen.at_end()),
    );

    loop {
        let dt = get_frame_time();
        let screen = get_screen_size(screen_width(), screen_height());
        world.insert_resource(screen);
        world.get_resource_mut::<Time>().unwrap().update(dt);

        schedule.run(&mut world);

        next_frame().await;
    }
}
