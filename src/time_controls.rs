use bevy::prelude::*;

use crate::GameState;

pub struct TimeControlsPlugin;

impl Plugin for TimeControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationSpeed>()
            .add_systems(Startup, setup_fixed_timestep)
            .add_systems(Update, (toggle_pause, change_speed, apply_speed));
    }
}

/// Base ticks per second for the simulation
const BASE_TICKS_PER_SECOND: f64 = 10.0;

#[derive(Resource)]
pub struct SimulationSpeed {
    pub multiplier: f32,
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

/// Set up the initial fixed timestep
fn setup_fixed_timestep(mut time: ResMut<Time<Fixed>>) {
    time.set_timestep_hz(BASE_TICKS_PER_SECOND);
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match current_state.get() {
            GameState::Running => {
                next_state.set(GameState::Paused);
                time.pause();
                info!("Paused");
            }
            GameState::Paused => {
                next_state.set(GameState::Running);
                time.unpause();
                info!("Resumed");
            }
        }
    }
}

fn change_speed(keyboard: Res<ButtonInput<KeyCode>>, mut speed: ResMut<SimulationSpeed>) {
    let old_speed = speed.multiplier;

    // Minus key (-) to slow down
    if keyboard.just_pressed(KeyCode::Minus) {
        speed.multiplier = (speed.multiplier - 0.25).max(0.25);
    }

    // Equals key (=) to speed up
    if keyboard.just_pressed(KeyCode::Equal) {
        speed.multiplier = (speed.multiplier + 0.25).min(4.0);
    }

    if speed.multiplier != old_speed {
        info!("Speed: {:.2}x", speed.multiplier);
    }
}

/// Apply the speed multiplier to the fixed timestep
fn apply_speed(speed: Res<SimulationSpeed>, mut time: ResMut<Time<Fixed>>) {
    if speed.is_changed() {
        let hz = BASE_TICKS_PER_SECOND * speed.multiplier as f64;
        time.set_timestep_hz(hz);
    }
}
