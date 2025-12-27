use bevy::prelude::*;

use crate::GameState;

pub struct TimeControlsPlugin;

impl Plugin for TimeControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationSpeed>()
            .add_systems(Update, (toggle_pause, change_speed));
    }
}

#[derive(Resource)]
pub struct SimulationSpeed {
    pub multiplier: f32,
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match current_state.get() {
            GameState::Running => {
                next_state.set(GameState::Paused);
                info!("Paused");
            }
            GameState::Paused => {
                next_state.set(GameState::Running);
                info!("Resumed");
            }
        }
    }
}

fn change_speed(keyboard: Res<ButtonInput<KeyCode>>, mut speed: ResMut<SimulationSpeed>) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        speed.multiplier = 1.0;
        info!("Speed: 1x");
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        speed.multiplier = 2.0;
        info!("Speed: 2x");
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        speed.multiplier = 4.0;
        info!("Speed: 4x");
    }
}
