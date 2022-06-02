mod game;

use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::ecs::query::Fetch;
use bevy::input::mouse::MouseMotion;
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::window::WindowDescriptor;
use crate::game::GamePlugin;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum GameState {
    Menu, Game
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.1, 0.4, 0.6)))
        .insert_resource(WindowDescriptor {
            title: "Voxel Game".to_string(),
            width: 1280.,
            height: 720.,
            cursor_locked: true,
            cursor_visible: false,
            ..default()
        })

        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(GamePlugin)

        .add_startup_system(setup)
        .add_state(GameState::Game)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

}

#[derive(Component)]
struct CameraController {
    euler_rotation: Vec3
}

fn camera_controller(
    time: Res<Time>,
    inputs: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController)>
) {
    let (mut camera, mut controller): (Mut<Transform>, Mut<CameraController>) = query.single_mut();

    let mut movement = Vec3::new(0.,0.,0.);

    if inputs.pressed(KeyCode::W) { movement.z += time.delta_seconds() }
    if inputs.pressed(KeyCode::S) { movement.z -= time.delta_seconds() }
    if inputs.pressed(KeyCode::A) { movement.x -= time.delta_seconds() }
    if inputs.pressed(KeyCode::D) { movement.x += time.delta_seconds() }

    if movement.x != 0. || movement.z != 0. { movement = movement.normalize() }

    for ev in mouse.iter() {
        let delta: Vec2 = ev.delta;
        controller.euler_rotation.x -= delta.y;
        controller.euler_rotation.y += delta.x;
        camera.rotation = Quat::from_euler(EulerRot::XYZ, 0., controller.euler_rotation.y.to_radians(), 0.) * Quat::from_euler(EulerRot::XYZ, controller.euler_rotation.x.to_radians(), 0., 0.);
    }
    let mut new_translation = camera.forward() * movement.z + camera.right() * movement.x;
    new_translation.y = 0.;
    camera.translation += new_translation;
}

