use std::fmt::format;
use bevy::prelude::*;
use crate::shape::Cube;
use crate::{MouseMotion, vec3};

#[derive(Bundle)]
struct PlayerBundle {
    #[bundle]
    model: PbrBundle,
    controller: PlayerController
}

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
pub struct PlayerController {
    pitch: f32,
    yaw: f32
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            pitch: 0.,
            yaw: 0.,
        }
    }
}

/* Setups a player entity and adds a pbr bundle as a component, then adds a camera as a child */
pub fn setup_player(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn_bundle(
        PlayerBundle {
            model: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule { ..default() })),
                material: materials.add(Color::rgb(0.4, 0.4, 0.4).into()),
                ..default()
            },
            controller: PlayerController::default()
        }
    ).with_children(|parent| {
        parent.spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::identity().with_translation(vec3(0., 3., 10.)),
            ..default()
        });
    });
}

pub fn update_controller(
    time: Res<Time>,
    inputs: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut PlayerController)>
) {
    let (mut transform, mut controller): (Mut<Transform>, Mut<PlayerController>) = query.single_mut();

    let mut movement = Vec3::new(0.,0.,0.);

    if inputs.pressed(KeyCode::W) { movement.z += 1. }
    if inputs.pressed(KeyCode::S) { movement.z -= 1. }
    if inputs.pressed(KeyCode::A) { movement.x -= 1. }
    if inputs.pressed(KeyCode::D) { movement.x += 1. }

    for ev in mouse.iter() {
        let delta: Vec2 = ev.delta;
        controller.yaw -= delta.y;
        controller.pitch += delta.x;
        transform.rotation = Quat::from_euler(EulerRot::XYZ, 0., controller.pitch.to_radians(), 0.) * Quat::from_euler(EulerRot::XYZ, controller.yaw.to_radians(), 0., 0.);
    }

    let mut speed = 5.;
    if inputs.pressed(KeyCode::LControl) { speed = 100.; }
    if inputs.pressed(KeyCode::Space) { transform.translation.y += time.delta_seconds() * speed }
    if inputs.pressed(KeyCode::LShift) { transform.translation.y -= time.delta_seconds() * speed }

    if movement.x == 0. && movement.z == 0. {
        return;
    }

    let mut new_translation = transform.forward() * movement.z + transform.right() * movement.x;
    new_translation.y = 0.;
    new_translation = new_translation.normalize() * time.delta_seconds() * speed; // TODO: SPEED HARD CODED FIX LATER
    info!("{} {} {}", new_translation.x, transform.translation.y, new_translation.z);
    transform.translation += new_translation;

}

pub fn debug_player(query: Query<&Transform, With<PlayerController>>) {
    let transform: &Transform = query.single();
    info!("{} {} {}", transform.translation.x, transform.translation.y, transform.translation.z);
}