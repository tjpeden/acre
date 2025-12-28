//! Pheromone system for ant communication and player control.
//!
//! Pheromones are chemical signals that influence ant behavior.
//! Players place pheromones to guide the colony.

use bevy::prelude::*;

use crate::GameState;
use crate::sprites;
use crate::world::{CurrentZLevel, TILE_SIZE, WORLD_SIZE};

pub struct PheromonePlugin;

impl Plugin for PheromonePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PheromoneGrids>()
            .init_resource::<SelectedPheromoneType>()
            .add_systems(Startup, spawn_pheromone_overlay)
            .add_systems(
                Update,
                (
                    pheromone_input,
                    update_pheromone_overlay,
                    cycle_pheromone_type,
                ),
            )
            .add_systems(
                FixedUpdate,
                pheromone_decay.run_if(in_state(GameState::Running)),
            );
    }
}

// ============================================================================
// Pheromone Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PheromoneType {
    #[default]
    Dig, // Attract diggers
    Forage, // Attract foragers toward leaves
    Home,   // Trail back to nest
    Avoid,  // Keep ants away
}

impl PheromoneType {
    pub fn color(&self) -> Color {
        match self {
            PheromoneType::Dig => sprites::pheromones::DIG,
            PheromoneType::Forage => sprites::pheromones::FORAGE,
            PheromoneType::Home => sprites::pheromones::HOME,
            PheromoneType::Avoid => sprites::pheromones::AVOID,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PheromoneType::Dig => "Dig",
            PheromoneType::Forage => "Forage",
            PheromoneType::Home => "Home",
            PheromoneType::Avoid => "Avoid",
        }
    }
}

// ============================================================================
// Resources
// ============================================================================

/// Storage for all pheromone grids
#[derive(Resource)]
pub struct PheromoneGrids {
    pub dig: Box<[[[f32; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]>,
    pub forage: Box<[[[f32; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]>,
    pub home: Box<[[[f32; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]>,
    pub avoid: Box<[[[f32; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]>,
}

impl Default for PheromoneGrids {
    fn default() -> Self {
        Self {
            dig: Box::new([[[0.0; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]),
            forage: Box::new([[[0.0; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]),
            home: Box::new([[[0.0; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]),
            avoid: Box::new([[[0.0; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]),
        }
    }
}

impl PheromoneGrids {
    /// Get the intensity of a pheromone type at a position
    pub fn get(&self, ptype: PheromoneType, x: usize, y: usize, z: usize) -> f32 {
        match ptype {
            PheromoneType::Dig => self.dig[z][y][x],
            PheromoneType::Forage => self.forage[z][y][x],
            PheromoneType::Home => self.home[z][y][x],
            PheromoneType::Avoid => self.avoid[z][y][x],
        }
    }

    /// Set the intensity of a pheromone type at a position
    pub fn set(&mut self, ptype: PheromoneType, x: usize, y: usize, z: usize, value: f32) {
        let grid = match ptype {
            PheromoneType::Dig => &mut self.dig,
            PheromoneType::Forage => &mut self.forage,
            PheromoneType::Home => &mut self.home,
            PheromoneType::Avoid => &mut self.avoid,
        };
        grid[z][y][x] = value.clamp(0.0, 1.0);
    }

    /// Add to the intensity of a pheromone type at a position
    pub fn add(&mut self, ptype: PheromoneType, x: usize, y: usize, z: usize, amount: f32) {
        let current = self.get(ptype, x, y, z);
        self.set(ptype, x, y, z, current + amount);
    }
}

/// Currently selected pheromone type for placement
#[derive(Resource, Default)]
pub struct SelectedPheromoneType(pub PheromoneType);

// ============================================================================
// Components
// ============================================================================

/// Marker for pheromone overlay sprites
#[derive(Component)]
pub struct PheromoneOverlay {
    pub x: usize,
    pub y: usize,
}

// ============================================================================
// Systems
// ============================================================================

/// Spawn overlay sprites for pheromone visualization
fn spawn_pheromone_overlay(mut commands: Commands) {
    for y in 0..WORLD_SIZE {
        for x in 0..WORLD_SIZE {
            let world_x = (x as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
            let world_y = (y as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;

            commands.spawn((
                Sprite {
                    color: Color::NONE,
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, 0.5), // Between tiles (0) and ants (1)
                PheromoneOverlay { x, y },
                Visibility::Hidden,
            ));
        }
    }
}

/// Update pheromone overlay colors based on current z-level
fn update_pheromone_overlay(
    pheromones: Res<PheromoneGrids>,
    current_z: Res<CurrentZLevel>,
    mut query: Query<(&PheromoneOverlay, &mut Sprite, &mut Visibility)>,
) {
    let z = current_z.0;

    for (overlay, mut sprite, mut visibility) in &mut query {
        let x = overlay.x;
        let y = overlay.y;

        // Get all pheromone values at this tile
        let dig = pheromones.dig[z][y][x];
        let forage = pheromones.forage[z][y][x];
        let home = pheromones.home[z][y][x];
        let avoid = pheromones.avoid[z][y][x];

        // Find the strongest pheromone
        let max_value = dig.max(forage).max(home).max(avoid);

        if max_value > 0.01 {
            *visibility = Visibility::Visible;

            // Blend colors based on relative intensities
            let total = dig + forage + home + avoid;
            if total > 0.0 {
                let dig_color = sprites::pheromones::DIG;
                let forage_color = sprites::pheromones::FORAGE;
                let home_color = sprites::pheromones::HOME;
                let avoid_color = sprites::pheromones::AVOID;

                // Weighted blend
                let r = (color_r(dig_color) * dig
                    + color_r(forage_color) * forage
                    + color_r(home_color) * home
                    + color_r(avoid_color) * avoid)
                    / total;
                let g = (color_g(dig_color) * dig
                    + color_g(forage_color) * forage
                    + color_g(home_color) * home
                    + color_g(avoid_color) * avoid)
                    / total;
                let b = (color_b(dig_color) * dig
                    + color_b(forage_color) * forage
                    + color_b(home_color) * home
                    + color_b(avoid_color) * avoid)
                    / total;

                sprite.color = Color::srgba(r, g, b, max_value * 0.6);
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

// Helper functions to extract color components
fn color_r(c: Color) -> f32 {
    match c {
        Color::Srgba(srgba) => srgba.red,
        _ => 0.5,
    }
}

fn color_g(c: Color) -> f32 {
    match c {
        Color::Srgba(srgba) => srgba.green,
        _ => 0.5,
    }
}

fn color_b(c: Color) -> f32 {
    match c {
        Color::Srgba(srgba) => srgba.blue,
        _ => 0.5,
    }
}

/// Decay all pheromones over time
fn pheromone_decay(mut pheromones: ResMut<PheromoneGrids>) {
    const DECAY_RATE: f32 = 0.0005; // Per tick - slow decay for persistent trails

    for z in 0..WORLD_SIZE {
        for y in 0..WORLD_SIZE {
            for x in 0..WORLD_SIZE {
                if pheromones.dig[z][y][x] > 0.0 {
                    pheromones.dig[z][y][x] = (pheromones.dig[z][y][x] - DECAY_RATE).max(0.0);
                }
                if pheromones.forage[z][y][x] > 0.0 {
                    pheromones.forage[z][y][x] = (pheromones.forage[z][y][x] - DECAY_RATE).max(0.0);
                }
                if pheromones.home[z][y][x] > 0.0 {
                    pheromones.home[z][y][x] = (pheromones.home[z][y][x] - DECAY_RATE).max(0.0);
                }
                if pheromones.avoid[z][y][x] > 0.0 {
                    pheromones.avoid[z][y][x] = (pheromones.avoid[z][y][x] - DECAY_RATE).max(0.0);
                }
            }
        }
    }
}

/// Handle player pheromone placement via mouse click
fn pheromone_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    current_z: Res<CurrentZLevel>,
    selected_type: Res<SelectedPheromoneType>,
    mut pheromones: ResMut<PheromoneGrids>,
) {
    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Convert screen position to world position
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Convert world position to grid position
    let grid_x = ((world_pos.x / TILE_SIZE) + (WORLD_SIZE as f32 / 2.0)).floor() as i32;
    let grid_y = ((world_pos.y / TILE_SIZE) + (WORLD_SIZE as f32 / 2.0)).floor() as i32;

    // Bounds check
    if grid_x < 0 || grid_x >= WORLD_SIZE as i32 || grid_y < 0 || grid_y >= WORLD_SIZE as i32 {
        return;
    }

    let x = grid_x as usize;
    let y = grid_y as usize;
    let z = current_z.0;

    // Add pheromone at this location
    pheromones.add(selected_type.0, x, y, z, 0.1);
}

/// Cycle through pheromone types with Tab key
fn cycle_pheromone_type(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedPheromoneType>,
) {
    if keyboard.just_pressed(KeyCode::Tab) {
        selected.0 = match selected.0 {
            PheromoneType::Dig => PheromoneType::Forage,
            PheromoneType::Forage => PheromoneType::Home,
            PheromoneType::Home => PheromoneType::Avoid,
            PheromoneType::Avoid => PheromoneType::Dig,
        };
        info!("Selected pheromone: {}", selected.0.name());
    }
}
