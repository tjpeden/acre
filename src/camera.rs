use bevy::prelude::*;

use crate::world::{CurrentZLevel, SURFACE_LEVEL, WORLD_SIZE};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, camera_pan)
            .add_systems(Update, camera_zoom)
            .add_systems(Update, camera_z_level);
    }
}

const PAN_SPEED: f32 = 500.0;
const ZOOM_SPEED: f32 = 0.1;
const MIN_SCALE: f32 = 0.5;
const MAX_SCALE: f32 = 5.0;

#[derive(Component)]
struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn camera_pan(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Projection), With<MainCamera>>,
) {
    let Ok((mut transform, projection)) = query.single_mut() else {
        return;
    };

    let scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        let movement = direction * PAN_SPEED * scale * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
    }
}

fn camera_zoom(
    mut scroll_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<&mut Projection, With<MainCamera>>,
) {
    let Ok(mut projection) = query.single_mut() else {
        return;
    };

    for event in scroll_events.read() {
        if let Projection::Orthographic(ref mut ortho) = *projection {
            let zoom_delta = -event.y * ZOOM_SPEED;
            ortho.scale = (ortho.scale + zoom_delta).clamp(MIN_SCALE, MAX_SCALE);
        }
    }
}

fn camera_z_level(keyboard: Res<ButtonInput<KeyCode>>, mut current_z: ResMut<CurrentZLevel>) {
    let go_up =
        keyboard.just_pressed(KeyCode::BracketRight) || keyboard.just_pressed(KeyCode::Period);
    let go_down =
        keyboard.just_pressed(KeyCode::BracketLeft) || keyboard.just_pressed(KeyCode::Comma);

    if go_up && current_z.0 < WORLD_SIZE - 1 {
        current_z.0 += 1;
        info!("Z-level: {} {}", current_z.0, z_level_label(current_z.0));
    }

    if go_down && current_z.0 > 0 {
        current_z.0 -= 1;
        info!("Z-level: {} {}", current_z.0, z_level_label(current_z.0));
    }
}

fn z_level_label(z: usize) -> &'static str {
    if z > SURFACE_LEVEL {
        "(above ground)"
    } else if z == SURFACE_LEVEL {
        "(surface)"
    } else {
        "(underground)"
    }
}
