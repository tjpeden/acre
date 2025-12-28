//! Minimal UI for displaying game state and colony stats.

use bevy::prelude::*;

use crate::GameState;
use crate::ants::{Ant, Caste};
use crate::pheromones::SelectedPheromoneType;
use crate::time_controls::SimulationSpeed;
use crate::world::{CurrentZLevel, FungusGarden, SURFACE_LEVEL};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, update_ui);
    }
}

// ============================================================================
// Components
// ============================================================================

/// Marker for the main UI root
#[derive(Component)]
struct UiRoot;

/// Marker for the status text (speed, pause state)
#[derive(Component)]
struct StatusText;

/// Marker for colony stats text
#[derive(Component)]
struct ColonyStatsText;

/// Marker for controls help text
#[derive(Component)]
struct ControlsText;

// ============================================================================
// Systems
// ============================================================================

fn setup_ui(mut commands: Commands) {
    // Root container - top-left corner
    commands
        .spawn((
            UiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Status line (speed, pause, z-level, pheromone)
            parent.spawn((
                StatusText,
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Colony stats
            parent.spawn((
                ColonyStatsText,
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.9, 0.8, 1.0)),
            ));

            // Controls help
            parent.spawn((
                ControlsText,
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 1.0)),
            ));
        });
}

fn update_ui(
    game_state: Res<State<GameState>>,
    speed: Res<SimulationSpeed>,
    current_z: Res<CurrentZLevel>,
    selected_pheromone: Res<SelectedPheromoneType>,
    fungus_garden: Res<FungusGarden>,
    ant_query: Query<&Caste, With<Ant>>,
    mut status_query: Query<
        &mut Text,
        (
            With<StatusText>,
            Without<ColonyStatsText>,
            Without<ControlsText>,
        ),
    >,
    mut colony_query: Query<
        &mut Text,
        (
            With<ColonyStatsText>,
            Without<StatusText>,
            Without<ControlsText>,
        ),
    >,
    mut controls_query: Query<
        &mut Text,
        (
            With<ControlsText>,
            Without<StatusText>,
            Without<ColonyStatsText>,
        ),
    >,
) {
    // Count ants by caste
    let mut queen_count = 0;
    let mut forager_count = 0;
    let mut gardener_count = 0;
    let mut soldier_count = 0;

    for caste in &ant_query {
        match caste {
            Caste::Queen => queen_count += 1,
            Caste::Forager => forager_count += 1,
            Caste::Gardener => gardener_count += 1,
            Caste::Soldier => soldier_count += 1,
        }
    }

    let total_ants = queen_count + forager_count + gardener_count + soldier_count;

    // Calculate z-level relative to surface
    let z_relative = current_z.0 as i32 - SURFACE_LEVEL as i32;
    let z_display = if z_relative == 0 {
        "Surface".to_string()
    } else if z_relative > 0 {
        format!("+{} (above)", z_relative)
    } else {
        format!("{} (below)", z_relative)
    };

    // Update status text
    if let Ok(mut text) = status_query.single_mut() {
        let pause_state = match game_state.get() {
            GameState::Running => "",
            GameState::Paused => " [PAUSED]",
        };

        **text = format!(
            "Speed: {:.2}x{}  |  Z: {}  |  Pheromone: {}",
            speed.multiplier,
            pause_state,
            z_display,
            selected_pheromone.0.name()
        );
    }

    // Update colony stats
    if let Ok(mut text) = colony_query.single_mut() {
        **text = format!(
            "Colony: {} ants (Q:{} F:{} G:{})\nGarden: {} food | {} mulch | {} leaves",
            total_ants,
            queen_count,
            forager_count,
            gardener_count,
            fungus_garden.food,
            fungus_garden.mulch,
            fungus_garden.leaves
        );
    }

    // Update controls help
    if let Ok(mut text) = controls_query.single_mut() {
        **text = "Space:Pause  -/=:Speed  []:Z-Level  Tab:Pheromone  Click:Place".to_string();
    }
}
