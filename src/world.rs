use bevy::prelude::*;

use crate::sprites;

pub const WORLD_SIZE: usize = 64;
pub const SURFACE_LEVEL: usize = 48;
pub const TILE_SIZE: f32 = 16.0;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGrid>()
            .init_resource::<CurrentZLevel>()
            .add_systems(Startup, spawn_tile_sprites)
            .add_systems(Update, update_tile_sprites);
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
