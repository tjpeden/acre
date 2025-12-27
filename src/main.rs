use bevy::prelude::*;

mod camera;
mod sprites;
mod time_controls;
mod world;

use camera::CameraPlugin;
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
        .add_plugins((WorldPlugin, CameraPlugin, TimeControlsPlugin))
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Running,
    Paused,
}
