use bevy::prelude::*;

mod ants;
mod camera;
mod pheromones;
mod sprites;
mod time_controls;
mod world;

use ants::AntPlugin;
use camera::CameraPlugin;
use pheromones::PheromonePlugin;
use time_controls::TimeControlsPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Acre - Ant Colony Simulation".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins((
            WorldPlugin,
            CameraPlugin,
            TimeControlsPlugin,
            AntPlugin,
            PheromonePlugin,
        ))
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Running,
    Paused,
}
