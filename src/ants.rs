//! Ant entities, components, and behaviors.

use bevy::prelude::*;

use crate::pheromones::{PheromoneGrids, PheromoneType};
use crate::sprites;
use crate::world::{
    CurrentZLevel, FungusGarden, LeafSource, SURFACE_LEVEL, TILE_SIZE, TileKind, Tree, WORLD_SIZE,
    WorldGrid,
};

pub struct AntPlugin;

impl Plugin for AntPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NestLocation>()
            .add_systems(Startup, spawn_founding_colony)
            .add_systems(Update, (update_ant_sprites, debug_spawn_ant))
            .add_systems(
                FixedUpdate,
                (
                    ant_behavior,
                    ant_digging,
                    ant_foraging,
                    ant_carrying,
                    ant_gardening,
                    ant_hunger,
                    ant_feeding,
                    ant_starvation,
                )
                    .chain(),
            );
    }
}

/// The location of the nest (where ants bring resources)
#[derive(Resource)]
pub struct NestLocation {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl Default for NestLocation {
    fn default() -> Self {
        let center = WORLD_SIZE / 2;
        Self {
            x: center,
            y: center,
            z: SURFACE_LEVEL,
        }
    }
}

// ============================================================================
// Components
// ============================================================================

/// Marker component for all ants
#[derive(Component)]
pub struct Ant;

/// Position in the world grid (tile coordinates)
#[derive(Component, Clone, Copy)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

/// The caste/role of an ant
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Caste {
    Queen,
    Forager,
    Gardener,
    Soldier,
}

impl Caste {
    pub fn color(&self) -> Color {
        match self {
            Caste::Queen => sprites::ants::QUEEN,
            Caste::Forager => sprites::ants::FORAGER,
            Caste::Gardener => sprites::ants::GARDENER,
            Caste::Soldier => sprites::ants::SOLDIER,
        }
    }

    pub fn size(&self) -> f32 {
        match self {
            Caste::Queen => sprites::ants::QUEEN_SIZE,
            Caste::Forager => sprites::ants::FORAGER_SIZE,
            Caste::Gardener => sprites::ants::GARDENER_SIZE,
            Caste::Soldier => sprites::ants::SOLDIER_SIZE,
        }
    }
}

/// Hunger level - ants die if this reaches max
#[derive(Component)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 100.0,
        }
    }
}

/// Age in simulation ticks
#[derive(Component, Default)]
pub struct Age(pub u32);

/// What the ant is currently carrying
#[derive(Component, Default)]
pub enum Carrying {
    #[default]
    Nothing,
    Leaf,
    Mulch,
    FungusFood,
}

/// Current task/behavior
#[derive(Component, Default)]
pub enum Task {
    #[default]
    Idle,
    Wandering,
    Digging {
        target_x: usize,
        target_y: usize,
        target_z: usize,
    },
    /// Moving toward a leaf source to cut leaves
    Foraging {
        target_tree: Entity,
    },
    /// Carrying a leaf back to the nest/garden
    CarryingHome {
        home_x: usize,
        home_y: usize,
        home_z: usize,
    },
    Gardening,
    /// Going to nest to eat
    SeekingFood,
}

// ============================================================================
// Systems
// ============================================================================

/// Spawn the founding queen and initial workers at the center of the surface
fn spawn_founding_colony(mut commands: Commands) {
    let center = WORLD_SIZE / 2;
    let surface_z = crate::world::SURFACE_LEVEL;

    // Spawn queen
    spawn_ant(&mut commands, center, center, surface_z, Caste::Queen);
    info!(
        "Founding queen spawned at ({}, {}, {})",
        center, center, surface_z
    );

    // Spawn foragers
    for i in 0..3 {
        spawn_ant(
            &mut commands,
            center + i + 1,
            center,
            surface_z,
            Caste::Forager,
        );
    }
    info!("Spawned 3 initial forager workers");

    // Spawn gardeners
    for i in 0..2 {
        spawn_ant(
            &mut commands,
            center - i - 1,
            center,
            surface_z,
            Caste::Gardener,
        );
    }
    info!("Spawned 2 initial gardener workers");
}

/// Spawn a single ant at the given grid position
fn spawn_ant(commands: &mut Commands, x: usize, y: usize, z: usize, caste: Caste) {
    let world_x = (x as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
    let world_y = (y as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;

    commands.spawn((
        Ant,
        GridPosition { x, y, z },
        caste,
        Hunger::default(),
        Age::default(),
        Carrying::Nothing,
        Task::Idle,
        Sprite {
            color: caste.color(),
            custom_size: Some(Vec2::splat(caste.size())),
            ..default()
        },
        Transform::from_xyz(world_x, world_y, 1.0),
    ));
}

/// Debug: spawn workers with F key
fn debug_spawn_ant(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    queen_query: Query<&GridPosition, With<Ant>>,
) {
    if keyboard.just_pressed(KeyCode::KeyF) {
        // Find queen position (or any ant if no queen)
        if let Some(pos) = queen_query.iter().next() {
            spawn_ant(&mut commands, pos.x, pos.y, pos.z, Caste::Forager);
            info!(
                "Debug: Spawned forager at ({}, {}, {})",
                pos.x, pos.y, pos.z
            );
        }
    }
}

/// Update ant sprite visibility and position based on current z-level
fn update_ant_sprites(
    current_z: Res<CurrentZLevel>,
    mut query: Query<(&GridPosition, &mut Transform, &mut Visibility), With<Ant>>,
) {
    for (grid_pos, mut transform, mut visibility) in &mut query {
        // Update world position from grid position
        let world_x = (grid_pos.x as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
        let world_y = (grid_pos.y as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
        transform.translation.x = world_x;
        transform.translation.y = world_y;

        // Only visible if on current z-level
        *visibility = if grid_pos.z == current_z.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Basic ant movement - wander randomly for now
fn ant_behavior(
    mut query: Query<(&mut GridPosition, &Caste, &mut Task, &Carrying), With<Ant>>,
    world_grid: Res<WorldGrid>,
    mut pheromones: ResMut<PheromoneGrids>,
    tree_query: Query<(Entity, &Tree, &LeafSource)>,
    fungus_garden: Res<FungusGarden>,
    nest_location: Res<NestLocation>,
) {
    for (mut grid_pos, caste, mut task, carrying) in &mut query {
        // Queen doesn't move (for now)
        if *caste == Caste::Queen {
            continue;
        }

        // Skip ants that are carrying things or already foraging/carrying home
        if !matches!(*carrying, Carrying::Nothing) {
            continue;
        }

        match *task {
            Task::Idle => {
                // Gardeners prioritize processing leaves at the nest
                if *caste == Caste::Gardener && fungus_garden.leaves > 0 {
                    // Check if at nest
                    if grid_pos.x == nest_location.x
                        && grid_pos.y == nest_location.y
                        && grid_pos.z == nest_location.z
                    {
                        *task = Task::Gardening;
                        continue;
                    } else {
                        // Go to nest to garden
                        *task = Task::CarryingHome {
                            home_x: nest_location.x,
                            home_y: nest_location.y,
                            home_z: nest_location.z,
                        };
                        continue;
                    }
                }

                // Foragers prioritize finding trees when there are Forage pheromones
                if *caste == Caste::Forager
                    && let Some(tree_entity) =
                        find_forage_target(&grid_pos, &pheromones, &tree_query)
                {
                    *task = Task::Foraging {
                        target_tree: tree_entity,
                    };
                    continue;
                }

                // Check for nearby dig pheromones
                if let Some((tx, ty, tz)) =
                    find_pheromone_dig_target(&grid_pos, &world_grid, &pheromones)
                {
                    *task = Task::Digging {
                        target_x: tx,
                        target_y: ty,
                        target_z: tz,
                    };
                    continue;
                }

                // Decide what to do randomly
                use rand::Rng;
                let mut rng = rand::rng();

                // Foragers: 30% forage, 10% dig, 60% wander
                // Gardeners: 50% go to garden (if leaves), 10% dig, 40% wander
                // Others: 10% dig, 90% wander
                if *caste == Caste::Forager && rng.random_ratio(3, 10) {
                    // Try to find a tree to forage
                    if let Some(tree_entity) = find_nearest_tree(&grid_pos, &tree_query) {
                        *task = Task::Foraging {
                            target_tree: tree_entity,
                        };
                    } else {
                        *task = Task::Wandering;
                    }
                } else if *caste == Caste::Gardener && rng.random_ratio(5, 10) {
                    // Gardeners go to nest to work
                    *task = Task::CarryingHome {
                        home_x: nest_location.x,
                        home_y: nest_location.y,
                        home_z: nest_location.z,
                    };
                } else if rng.random_ratio(1, 10) {
                    if let Some((tx, ty, tz)) = find_diggable_tile(&grid_pos, &world_grid) {
                        *task = Task::Digging {
                            target_x: tx,
                            target_y: ty,
                            target_z: tz,
                        };
                    } else {
                        *task = Task::Wandering;
                    }
                } else {
                    *task = Task::Wandering;
                }
            }
            Task::Wandering => {
                // Check for pheromones to follow and reinforce trails
                try_pheromone_biased_move(&mut grid_pos, &world_grid, &mut pheromones);

                // Small chance to go idle and reconsider
                use rand::Rng;
                let mut rng = rand::rng();
                if rng.random_ratio(1, 30) {
                    *task = Task::Idle;
                }
            }
            Task::Digging {
                target_x,
                target_y,
                target_z,
            } => {
                // Move towards target if not adjacent
                let dx = (target_x as i32 - grid_pos.x as i32).signum();
                let dy = (target_y as i32 - grid_pos.y as i32).signum();
                let dz = (target_z as i32 - grid_pos.z as i32).signum();

                // Check if we're adjacent to the target (including z)
                let dist_x = (target_x as i32 - grid_pos.x as i32).abs();
                let dist_y = (target_y as i32 - grid_pos.y as i32).abs();
                let dist_z = (target_z as i32 - grid_pos.z as i32).abs();

                let is_adjacent =
                    (dist_x <= 1 && dist_y <= 1 && dist_z <= 1) && (dist_x + dist_y + dist_z > 0);

                if is_adjacent {
                    // We're adjacent - digging happens in ant_digging system
                    // Stay in Digging state
                } else {
                    // Move towards target on same z-level first
                    if dist_x > 0 || dist_y > 0 {
                        let new_x =
                            (grid_pos.x as i32 + dx).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                        let new_y =
                            (grid_pos.y as i32 + dy).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                        let tile = world_grid.tiles[grid_pos.z][new_y][new_x];
                        if is_passable(tile) {
                            grid_pos.x = new_x;
                            grid_pos.y = new_y;
                        }
                    } else if dist_z > 0 && dz < 0 {
                        // Need to go down - check if tile below is passable
                        let new_z =
                            (grid_pos.z as i32 + dz).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                        let tile = world_grid.tiles[new_z][grid_pos.y][grid_pos.x];
                        if is_passable(tile) {
                            grid_pos.z = new_z;
                        }
                    }
                }
            }
            Task::Foraging { .. } | Task::CarryingHome { .. } | Task::SeekingFood => {
                // Handled by ant_foraging, ant_carrying, and ant_feeding systems
            }
            Task::Gardening => {
                // Handled by ant_gardening system
            }
        }
    }
}

/// Find a dirt tile adjacent to the ant that can be dug
fn find_diggable_tile(pos: &GridPosition, world_grid: &WorldGrid) -> Option<(usize, usize, usize)> {
    // Priority: check below first, then cardinal directions on same level
    let candidates = [
        (0, 0, -1), // below
        (1, 0, 0),  // east
        (-1, 0, 0), // west
        (0, 1, 0),  // north
        (0, -1, 0), // south
    ];

    for (dx, dy, dz) in candidates {
        let nx = pos.x as i32 + dx;
        let ny = pos.y as i32 + dy;
        let nz = pos.z as i32 + dz;

        if nx < 0
            || nx >= WORLD_SIZE as i32
            || ny < 0
            || ny >= WORLD_SIZE as i32
            || nz < 0
            || nz >= WORLD_SIZE as i32
        {
            continue;
        }

        let tile = world_grid.tiles[nz as usize][ny as usize][nx as usize];
        if tile == TileKind::Dirt {
            return Some((nx as usize, ny as usize, nz as usize));
        }
    }

    None
}

/// System that performs actual digging
fn ant_digging(
    mut query: Query<(&GridPosition, &mut Task), With<Ant>>,
    mut world_grid: ResMut<WorldGrid>,
) {
    for (grid_pos, mut task) in &mut query {
        if let Task::Digging {
            target_x,
            target_y,
            target_z,
        } = *task
        {
            // Check if we're adjacent to target
            let dist_x = (target_x as i32 - grid_pos.x as i32).abs();
            let dist_y = (target_y as i32 - grid_pos.y as i32).abs();
            let dist_z = (target_z as i32 - grid_pos.z as i32).abs();

            let is_adjacent =
                (dist_x <= 1 && dist_y <= 1 && dist_z <= 1) && (dist_x + dist_y + dist_z > 0);

            if is_adjacent {
                // Check if target is still dirt
                if world_grid.tiles[target_z][target_y][target_x] == TileKind::Dirt {
                    // Dig it!
                    world_grid.tiles[target_z][target_y][target_x] = TileKind::Tunnel;
                    info!(
                        "Ant dug tunnel at ({}, {}, {})",
                        target_x, target_y, target_z
                    );
                }
                // Task complete - go idle
                *task = Task::Idle;
            }
        }
    }
}

/// System that handles ants foraging for leaves from trees
fn ant_foraging(
    mut ant_query: Query<(&mut GridPosition, &mut Task, &mut Carrying), With<Ant>>,
    mut tree_query: Query<(&Tree, &mut LeafSource)>,
    world_grid: Res<WorldGrid>,
    nest_location: Res<NestLocation>,
    mut pheromones: ResMut<PheromoneGrids>,
) {
    for (mut grid_pos, mut task, mut carrying) in &mut ant_query {
        if let Task::Foraging { target_tree } = *task {
            // Get the tree's position
            let Some((tree, mut leaf_source)) = tree_query.get_mut(target_tree).ok() else {
                // Tree no longer exists, go idle
                *task = Task::Idle;
                continue;
            };

            // Check if tree still has leaves
            if leaf_source.leaves_remaining == 0 {
                *task = Task::Idle;
                continue;
            }

            let tree_x = tree.x;
            let tree_y = tree.y;

            // Check if we're adjacent to the tree (on the surface level)
            let dist_x = (tree_x as i32 - grid_pos.x as i32).abs();
            let dist_y = (tree_y as i32 - grid_pos.y as i32).abs();
            let is_adjacent = dist_x <= 1 && dist_y <= 1 && (dist_x + dist_y > 0);

            if is_adjacent && grid_pos.z == SURFACE_LEVEL {
                // We're next to the tree - cut a leaf!
                leaf_source.leaves_remaining = leaf_source.leaves_remaining.saturating_sub(1);
                *carrying = Carrying::Leaf;

                // Deposit strong Forage pheromone at this successful foraging location
                pheromones.add(
                    PheromoneType::Forage,
                    grid_pos.x,
                    grid_pos.y,
                    grid_pos.z,
                    0.3,
                );

                info!(
                    "Ant cut leaf from tree at ({}, {}). {} leaves remaining.",
                    tree_x, tree_y, leaf_source.leaves_remaining
                );

                // Now carry the leaf home
                *task = Task::CarryingHome {
                    home_x: nest_location.x,
                    home_y: nest_location.y,
                    home_z: nest_location.z,
                };
            } else {
                // Move towards the tree on the surface level
                if grid_pos.z != SURFACE_LEVEL {
                    // Need to get to surface first - move up if possible
                    let new_z = grid_pos.z + 1;
                    if new_z < WORLD_SIZE
                        && is_passable(world_grid.tiles[new_z][grid_pos.y][grid_pos.x])
                    {
                        grid_pos.z = new_z;
                    }
                } else {
                    // Move towards tree on surface
                    let dx = (tree_x as i32 - grid_pos.x as i32).signum();
                    let dy = (tree_y as i32 - grid_pos.y as i32).signum();

                    let new_x = (grid_pos.x as i32 + dx).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                    let new_y = (grid_pos.y as i32 + dy).clamp(0, WORLD_SIZE as i32 - 1) as usize;

                    if is_passable(world_grid.tiles[grid_pos.z][new_y][new_x]) {
                        grid_pos.x = new_x;
                        grid_pos.y = new_y;
                    } else if is_passable(world_grid.tiles[grid_pos.z][grid_pos.y][new_x]) {
                        // Try just x movement
                        grid_pos.x = new_x;
                    } else if is_passable(world_grid.tiles[grid_pos.z][new_y][grid_pos.x]) {
                        // Try just y movement
                        grid_pos.y = new_y;
                    }
                }
            }
        }
    }
}

/// System that handles ants carrying resources back to the nest
fn ant_carrying(
    mut query: Query<(&mut GridPosition, &mut Task, &mut Carrying), With<Ant>>,
    world_grid: Res<WorldGrid>,
    mut fungus_garden: ResMut<FungusGarden>,
    mut pheromones: ResMut<PheromoneGrids>,
) {
    for (mut grid_pos, mut task, mut carrying) in &mut query {
        if let Task::CarryingHome {
            home_x,
            home_y,
            home_z,
        } = *task
        {
            // Check if we're at the nest
            if grid_pos.x == home_x && grid_pos.y == home_y && grid_pos.z == home_z {
                // Drop the resource into the fungus garden
                if let Carrying::Leaf = *carrying {
                    fungus_garden.add_leaf();
                    info!(
                        "Ant delivered leaf to garden. Total: {} leaves, {} mulch, {} food",
                        fungus_garden.leaves, fungus_garden.mulch, fungus_garden.food
                    );
                }
                *carrying = Carrying::Nothing;
                *task = Task::Idle;
            } else {
                // Deposit Home pheromone while carrying resources back
                // This creates a trail for other ants to follow home
                if matches!(*carrying, Carrying::Leaf) {
                    pheromones.add(
                        PheromoneType::Home,
                        grid_pos.x,
                        grid_pos.y,
                        grid_pos.z,
                        0.05,
                    );
                }

                // Move towards home
                let dx = (home_x as i32 - grid_pos.x as i32).signum();
                let dy = (home_y as i32 - grid_pos.y as i32).signum();
                let dz = (home_z as i32 - grid_pos.z as i32).signum();

                // Try to move on the same z-level first
                if grid_pos.z == home_z || dz == 0 {
                    let new_x = (grid_pos.x as i32 + dx).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                    let new_y = (grid_pos.y as i32 + dy).clamp(0, WORLD_SIZE as i32 - 1) as usize;

                    if is_passable(world_grid.tiles[grid_pos.z][new_y][new_x]) {
                        grid_pos.x = new_x;
                        grid_pos.y = new_y;
                    } else if dx != 0
                        && is_passable(world_grid.tiles[grid_pos.z][grid_pos.y][new_x])
                    {
                        grid_pos.x = new_x;
                    } else if dy != 0
                        && is_passable(world_grid.tiles[grid_pos.z][new_y][grid_pos.x])
                    {
                        grid_pos.y = new_y;
                    }
                } else {
                    // Need to change z-level
                    let new_z = (grid_pos.z as i32 + dz).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                    if is_passable(world_grid.tiles[new_z][grid_pos.y][grid_pos.x]) {
                        grid_pos.z = new_z;
                    }
                }
            }
        }
    }
}

/// System that handles gardener ants processing leaves into mulch
fn ant_gardening(
    mut query: Query<(&GridPosition, &mut Task), With<Ant>>,
    mut fungus_garden: ResMut<FungusGarden>,
    nest_location: Res<NestLocation>,
) {
    for (grid_pos, mut task) in &mut query {
        if let Task::Gardening = *task {
            // Must be at the nest to garden
            if grid_pos.x == nest_location.x
                && grid_pos.y == nest_location.y
                && grid_pos.z == nest_location.z
            {
                // Try to process a leaf into mulch
                if fungus_garden.process_leaf() {
                    info!(
                        "Gardener processed leaf into mulch. Garden: {} leaves, {} mulch, {} food",
                        fungus_garden.leaves, fungus_garden.mulch, fungus_garden.food
                    );
                }

                // If no more leaves, go idle
                if fungus_garden.leaves == 0 {
                    *task = Task::Idle;
                }
                // Otherwise stay gardening
            } else {
                // Not at nest, go idle (ant_behavior will redirect us)
                *task = Task::Idle;
            }
        }
    }
}

/// Hunger constant: how much hunger increases per tick
const HUNGER_RATE: f32 = 0.15;
/// Hunger threshold at which ants will seek food
const HUNGER_THRESHOLD: f32 = 50.0;

/// System that increases ant hunger over time
fn ant_hunger(mut query: Query<(&mut Hunger, &mut Task, &Caste), With<Ant>>) {
    for (mut hunger, mut task, caste) in &mut query {
        // Queen gets hungry slower
        let rate = if *caste == Caste::Queen {
            HUNGER_RATE * 0.5
        } else {
            HUNGER_RATE
        };

        hunger.current += rate;

        // If very hungry and not already seeking food or doing critical task, go eat
        if hunger.current >= HUNGER_THRESHOLD {
            match *task {
                Task::SeekingFood | Task::CarryingHome { .. } => {
                    // Already heading home or seeking food
                }
                _ => {
                    // Drop everything and go eat
                    *task = Task::SeekingFood;
                }
            }
        }
    }
}

/// System that handles ants eating at the nest
fn ant_feeding(
    mut query: Query<(&mut GridPosition, &mut Hunger, &mut Task), With<Ant>>,
    mut fungus_garden: ResMut<FungusGarden>,
    nest_location: Res<NestLocation>,
    world_grid: Res<WorldGrid>,
) {
    for (mut grid_pos, mut hunger, mut task) in &mut query {
        if let Task::SeekingFood = *task {
            // Check if at nest
            if grid_pos.x == nest_location.x
                && grid_pos.y == nest_location.y
                && grid_pos.z == nest_location.z
            {
                // Try to eat
                if fungus_garden.consume_food() {
                    hunger.current = 0.0;
                    info!(
                        "Ant ate food. {} food remaining in garden.",
                        fungus_garden.food
                    );
                    *task = Task::Idle;
                }
                // If no food, stay seeking (will starve if too long)
            } else {
                // Move toward nest
                let home_x = nest_location.x;
                let home_y = nest_location.y;
                let home_z = nest_location.z;

                let dx = (home_x as i32 - grid_pos.x as i32).signum();
                let dy = (home_y as i32 - grid_pos.y as i32).signum();
                let dz = (home_z as i32 - grid_pos.z as i32).signum();

                // Try to move on the same z-level first
                if grid_pos.z == home_z || dz == 0 {
                    let new_x = (grid_pos.x as i32 + dx).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                    let new_y = (grid_pos.y as i32 + dy).clamp(0, WORLD_SIZE as i32 - 1) as usize;

                    if is_passable(world_grid.tiles[grid_pos.z][new_y][new_x]) {
                        grid_pos.x = new_x;
                        grid_pos.y = new_y;
                    } else if dx != 0
                        && is_passable(world_grid.tiles[grid_pos.z][grid_pos.y][new_x])
                    {
                        grid_pos.x = new_x;
                    } else if dy != 0
                        && is_passable(world_grid.tiles[grid_pos.z][new_y][grid_pos.x])
                    {
                        grid_pos.y = new_y;
                    }
                } else {
                    // Need to change z-level
                    let new_z = (grid_pos.z as i32 + dz).clamp(0, WORLD_SIZE as i32 - 1) as usize;
                    if is_passable(world_grid.tiles[new_z][grid_pos.y][grid_pos.x]) {
                        grid_pos.z = new_z;
                    }
                }
            }
        }
    }
}

/// System that kills ants that have starved
fn ant_starvation(mut commands: Commands, query: Query<(Entity, &Hunger, &Caste), With<Ant>>) {
    for (entity, hunger, caste) in &query {
        if hunger.current >= hunger.max {
            info!("A {:?} ant has starved to death!", caste);
            commands.entity(entity).despawn();
        }
    }
}

/// Move biased by pheromone gradients, with random fallback
/// Also reinforces pheromone trails when following them
fn try_pheromone_biased_move(
    grid_pos: &mut GridPosition,
    world_grid: &WorldGrid,
    pheromones: &mut PheromoneGrids,
) {
    use rand::Rng;

    let mut rng = rand::rng();
    let directions: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

    // Calculate weights for each direction based on pheromones
    let mut weights: [f32; 4] = [1.0; 4]; // Base weight of 1.0 for each direction
    let mut total_weight = 0.0;
    let mut pheromone_influence: [f32; 4] = [0.0; 4]; // Track pheromone contribution

    for (i, (dx, dy)) in directions.iter().enumerate() {
        let new_x = grid_pos.x as i32 + dx;
        let new_y = grid_pos.y as i32 + dy;

        // Skip invalid positions
        if new_x < 0 || new_x >= WORLD_SIZE as i32 || new_y < 0 || new_y >= WORLD_SIZE as i32 {
            weights[i] = 0.0;
            continue;
        }

        let nx = new_x as usize;
        let ny = new_y as usize;
        let z = grid_pos.z;

        // Check passability
        if !is_passable(world_grid.tiles[z][ny][nx]) {
            weights[i] = 0.0;
            continue;
        }

        // Add pheromone attraction (dig, forage, and home are attractive)
        let dig_strength = pheromones.get(PheromoneType::Dig, nx, ny, z);
        let forage_strength = pheromones.get(PheromoneType::Forage, nx, ny, z);
        let home_strength = pheromones.get(PheromoneType::Home, nx, ny, z);
        let avoid_strength = pheromones.get(PheromoneType::Avoid, nx, ny, z);

        // Track how much pheromone influenced this direction
        pheromone_influence[i] = dig_strength + forage_strength + home_strength;

        // Boost weight based on attractive pheromones
        weights[i] += dig_strength * 5.0 + forage_strength * 3.0 + home_strength * 2.0;

        // Reduce weight for avoid pheromones
        weights[i] *= 1.0 - (avoid_strength * 0.9);

        // Ensure non-negative
        weights[i] = weights[i].max(0.0);
        total_weight += weights[i];
    }

    // If no valid moves, return
    if total_weight <= 0.0 {
        return;
    }

    // Weighted random selection
    let mut roll = rng.random_range(0.0..total_weight);
    for (i, (dx, dy)) in directions.iter().enumerate() {
        roll -= weights[i];
        if roll <= 0.0 {
            let new_x = (grid_pos.x as i32 + dx) as usize;
            let new_y = (grid_pos.y as i32 + dy) as usize;

            // If this move was influenced by pheromones, reinforce the trail slightly
            // This creates positive feedback for successful paths
            if pheromone_influence[i] > 0.1 {
                let z = grid_pos.z;
                // Reinforce at the OLD position (where the ant just was)
                // This strengthens the path that led here
                let forage_at_new = pheromones.get(PheromoneType::Forage, new_x, new_y, z);
                let home_at_new = pheromones.get(PheromoneType::Home, new_x, new_y, z);

                if forage_at_new > 0.05 {
                    pheromones.add(PheromoneType::Forage, grid_pos.x, grid_pos.y, z, 0.01);
                }
                if home_at_new > 0.05 {
                    pheromones.add(PheromoneType::Home, grid_pos.x, grid_pos.y, z, 0.01);
                }
            }

            grid_pos.x = new_x;
            grid_pos.y = new_y;
            return;
        }
    }
}

/// Find a dirt tile to dig based on nearby dig pheromones
fn find_pheromone_dig_target(
    pos: &GridPosition,
    world_grid: &WorldGrid,
    pheromones: &PheromoneGrids,
) -> Option<(usize, usize, usize)> {
    // Search in a small radius for dig pheromones near dirt tiles
    let search_radius: i32 = 5;
    let mut best_target: Option<(usize, usize, usize)> = None;
    let mut best_score: f32 = 0.1; // Minimum threshold

    for dz in -1..=0 {
        // Check current level and one below
        for dy in -search_radius..=search_radius {
            for dx in -search_radius..=search_radius {
                let nx = pos.x as i32 + dx;
                let ny = pos.y as i32 + dy;
                let nz = pos.z as i32 + dz;

                if nx < 0
                    || nx >= WORLD_SIZE as i32
                    || ny < 0
                    || ny >= WORLD_SIZE as i32
                    || nz < 0
                    || nz >= WORLD_SIZE as i32
                {
                    continue;
                }

                let x = nx as usize;
                let y = ny as usize;
                let z = nz as usize;

                // Must be a dirt tile
                if world_grid.tiles[z][y][x] != TileKind::Dirt {
                    continue;
                }

                // Check dig pheromone strength
                let dig_strength = pheromones.get(PheromoneType::Dig, x, y, z);

                // Score based on pheromone strength and distance (prefer closer)
                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                let score = dig_strength / (1.0 + distance * 0.2);

                if score > best_score {
                    best_score = score;
                    best_target = Some((x, y, z));
                }
            }
        }
    }

    best_target
}

/// Check if a tile can be walked on
fn is_passable(tile: TileKind) -> bool {
    matches!(
        tile,
        TileKind::Surface | TileKind::Tunnel | TileKind::Chamber | TileKind::FungusGarden
    )
}

/// Find a tree to forage based on Forage pheromone presence
fn find_forage_target(
    pos: &GridPosition,
    pheromones: &PheromoneGrids,
    tree_query: &Query<(Entity, &Tree, &LeafSource)>,
) -> Option<Entity> {
    // Check if there's significant Forage pheromone nearby
    let search_radius: i32 = 5;
    let mut has_forage_pheromone = false;

    for dy in -search_radius..=search_radius {
        for dx in -search_radius..=search_radius {
            let nx = pos.x as i32 + dx;
            let ny = pos.y as i32 + dy;

            if nx < 0 || nx >= WORLD_SIZE as i32 || ny < 0 || ny >= WORLD_SIZE as i32 {
                continue;
            }

            let forage_strength =
                pheromones.get(PheromoneType::Forage, nx as usize, ny as usize, pos.z);
            if forage_strength > 0.1 {
                has_forage_pheromone = true;
                break;
            }
        }
        if has_forage_pheromone {
            break;
        }
    }

    if !has_forage_pheromone {
        return None;
    }

    // Find the nearest tree with leaves
    find_nearest_tree(pos, tree_query)
}

/// Find the nearest tree that has leaves remaining
fn find_nearest_tree(
    pos: &GridPosition,
    tree_query: &Query<(Entity, &Tree, &LeafSource)>,
) -> Option<Entity> {
    let mut best_tree: Option<Entity> = None;
    let mut best_distance = i32::MAX;

    for (entity, tree, leaf_source) in tree_query.iter() {
        // Skip trees with no leaves
        if leaf_source.leaves_remaining == 0 {
            continue;
        }

        let dist = (tree.x as i32 - pos.x as i32).abs() + (tree.y as i32 - pos.y as i32).abs();
        if dist < best_distance {
            best_distance = dist;
            best_tree = Some(entity);
        }
    }

    best_tree
}
