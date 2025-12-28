use bevy::prelude::*;
use rand::Rng;

use crate::sprites;

pub const WORLD_SIZE: usize = 64;
pub const SURFACE_LEVEL: usize = 48;
pub const TILE_SIZE: f32 = 16.0;
pub const TREE_HEIGHT: usize = 6; // Trunk + canopy

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGrid>()
            .init_resource::<CurrentZLevel>()
            .init_resource::<FungusGarden>()
            .add_systems(Startup, (init_world_with_trees, spawn_tile_sprites).chain())
            .add_systems(Update, update_tile_sprites)
            .add_systems(FixedUpdate, fungus_growth);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileKind {
    #[default]
    Air,
    Surface,
    Dirt,
    Tunnel,
    Chamber,
    FungusGarden,
    TreeTrunk,
    TreeCanopy,
}

impl TileKind {
    pub fn color(&self) -> Color {
        match self {
            TileKind::Air => sprites::tiles::AIR,
            TileKind::Surface => sprites::tiles::SURFACE,
            TileKind::Dirt => sprites::tiles::DIRT,
            TileKind::Tunnel => sprites::tiles::TUNNEL,
            TileKind::Chamber => sprites::tiles::CHAMBER,
            TileKind::FungusGarden => sprites::tiles::FUNGUS_GARDEN,
            TileKind::TreeTrunk => sprites::tiles::TREE_TRUNK,
            TileKind::TreeCanopy => sprites::tiles::TREE_CANOPY,
        }
    }
}

#[derive(Resource)]
pub struct WorldGrid {
    pub tiles: Box<[[[TileKind; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]>,
}

impl Default for WorldGrid {
    fn default() -> Self {
        let mut tiles = Box::new([[[TileKind::Air; WORLD_SIZE]; WORLD_SIZE]; WORLD_SIZE]);

        for z in 0..WORLD_SIZE {
            for y in 0..WORLD_SIZE {
                for x in 0..WORLD_SIZE {
                    tiles[z][y][x] = if z < SURFACE_LEVEL {
                        TileKind::Dirt
                    } else if z == SURFACE_LEVEL {
                        TileKind::Surface
                    } else {
                        TileKind::Air
                    };
                }
            }
        }

        Self { tiles }
    }
}

// ============================================================================
// Tree/Plant Components
// ============================================================================

/// Marker for a tree entity
#[derive(Component)]
pub struct Tree {
    pub x: usize,
    pub y: usize,
}

/// A leaf source that can be harvested
#[derive(Component)]
pub struct LeafSource {
    pub leaves_remaining: u32,
    pub max_leaves: u32,
    pub regrow_timer: f32,
}

impl Default for LeafSource {
    fn default() -> Self {
        Self {
            leaves_remaining: 20,
            max_leaves: 20,
            regrow_timer: 0.0,
        }
    }
}

// ============================================================================
// Fungus Garden Resource
// ============================================================================

/// The colony's fungus garden - stores leaves, mulch, and food
#[derive(Resource)]
pub struct FungusGarden {
    /// Raw leaves waiting to be processed
    pub leaves: u32,
    /// Mulch (processed leaves) that fungus grows on
    pub mulch: u32,
    /// Food available for ants to eat
    pub food: u32,
    /// Progress toward next food unit (0.0 - 1.0)
    pub growth_progress: f32,
}

impl Default for FungusGarden {
    fn default() -> Self {
        Self {
            leaves: 0,
            mulch: 0,
            food: 10, // Start with some food so colony doesn't immediately starve
            growth_progress: 0.0,
        }
    }
}

impl FungusGarden {
    /// Add a leaf to the garden (called when forager delivers)
    pub fn add_leaf(&mut self) {
        self.leaves += 1;
    }

    /// Gardener processes a leaf into mulch
    pub fn process_leaf(&mut self) -> bool {
        if self.leaves > 0 {
            self.leaves -= 1;
            self.mulch += 1;
            true
        } else {
            false
        }
    }

    /// Try to consume food (returns true if food was available)
    pub fn consume_food(&mut self) -> bool {
        if self.food > 0 {
            self.food -= 1;
            true
        } else {
            false
        }
    }
}

/// Fungus grows on mulch and produces food over time
fn fungus_growth(mut garden: ResMut<FungusGarden>) {
    // No mulch = no growth
    if garden.mulch == 0 {
        return;
    }

    // Growth rate scales with amount of mulch (diminishing returns)
    // Base rate: 0.01 per tick, boosted by sqrt(mulch)
    let growth_rate = 0.005 * (garden.mulch as f32).sqrt();
    garden.growth_progress += growth_rate;

    // When progress reaches 1.0, produce food and consume some mulch
    if garden.growth_progress >= 1.0 {
        garden.growth_progress -= 1.0;
        garden.food += 1;
        // Mulch slowly depletes as fungus consumes it
        if garden.mulch > 0 {
            garden.mulch -= 1;
        }
        info!(
            "Fungus produced food! Garden: {} leaves, {} mulch, {} food",
            garden.leaves, garden.mulch, garden.food
        );
    }
}

// ============================================================================
// Systems
// ============================================================================

/// Initialize the world with trees
fn init_world_with_trees(mut commands: Commands, mut world_grid: ResMut<WorldGrid>) {
    let mut rng = rand::rng();
    let num_trees = 8; // Start with a few trees

    for _ in 0..num_trees {
        // Random position, but not too close to center (where queen spawns)
        let x = rng.random_range(5..WORLD_SIZE - 5);
        let y = rng.random_range(5..WORLD_SIZE - 5);

        // Skip if too close to center
        let center = WORLD_SIZE / 2;
        if (x as i32 - center as i32).abs() < 8 && (y as i32 - center as i32).abs() < 8 {
            continue;
        }

        spawn_tree(&mut commands, &mut world_grid, x, y);
    }

    info!("Spawned trees in the world");
}

/// Spawn a tree at the given surface position
fn spawn_tree(commands: &mut Commands, world_grid: &mut WorldGrid, x: usize, y: usize) {
    let base_z = SURFACE_LEVEL + 1;

    // Create trunk (3 tiles high)
    for z_offset in 0..3 {
        let z = base_z + z_offset;
        if z < WORLD_SIZE {
            world_grid.tiles[z][y][x] = TileKind::TreeTrunk;
        }
    }

    // Create canopy (3 tiles high, with some spread)
    let canopy_base = base_z + 3;
    for z_offset in 0..3 {
        let z = canopy_base + z_offset;
        if z >= WORLD_SIZE {
            continue;
        }

        // Canopy spreads out
        let spread = if z_offset == 1 { 1 } else { 0 };
        for dy in -(spread as i32)..=(spread as i32) {
            for dx in -(spread as i32)..=(spread as i32) {
                let nx = (x as i32 + dx).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                let ny = (y as i32 + dy).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                world_grid.tiles[z][ny][nx] = TileKind::TreeCanopy;
            }
        }
    }

    // Spawn tree entity with leaf source at canopy level
    let canopy_z = canopy_base + 1;
    let world_x = (x as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
    let world_y = (y as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;

    commands.spawn((
        Tree { x, y },
        LeafSource::default(),
        Sprite {
            color: sprites::objects::LEAF_FRAGMENT,
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.5)),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 0.8),
        TreeCanopyMarker { z: canopy_z },
    ));
}

/// Marker to track which z-level the tree canopy is at (for visibility)
#[derive(Component)]
pub struct TreeCanopyMarker {
    pub z: usize,
}

#[derive(Resource)]
pub struct CurrentZLevel(pub usize);

impl Default for CurrentZLevel {
    fn default() -> Self {
        Self(SURFACE_LEVEL)
    }
}

#[derive(Component)]
pub struct TileSprite {
    pub x: usize,
    pub y: usize,
}

fn spawn_tile_sprites(mut commands: Commands) {
    // Spawn a sprite for each tile position in the current view
    for y in 0..WORLD_SIZE {
        for x in 0..WORLD_SIZE {
            let world_x = (x as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
            let world_y = (y as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;

            commands.spawn((
                Sprite {
                    color: Color::srgb(0.5, 0.5, 0.5),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(world_x, world_y, 0.0),
                TileSprite { x, y },
            ));
        }
    }
}

fn update_tile_sprites(
    world_grid: Res<WorldGrid>,
    current_z: Res<CurrentZLevel>,
    mut query: Query<(&TileSprite, &mut Sprite)>,
) {
    if !current_z.is_changed() && !world_grid.is_changed() {
        return;
    }

    let z = current_z.0;
    for (tile_sprite, mut sprite) in &mut query {
        let tile_kind = world_grid.tiles[z][tile_sprite.y][tile_sprite.x];
        sprite.color = tile_kind.color();
    }
}
