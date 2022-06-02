use std::future::Future;
use std::task::Poll;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;
use futures_lite::future;
use futures_lite::future::FutureExt;
use crate::game::player::PlayerController;
use crate::game::world::chunk;
use crate::game::world::chunk::{Chunk, CHUNK_LENGTH};
use crate::game::world::generator::TerrainGenerator;
use crate::game::world::voxel::VoxelType;

#[derive(Component)]
pub struct Terrain;

#[derive(Default)]
pub struct World {
    terrain_entity: Option<Entity>,
    terrain_material: Handle<StandardMaterial>,
    chunk_ledger: HashMap<(i32, i32), Chunk>,
    bevy_chunk_ledger: HashMap<(i32, i32), Entity>,
    loading_ledger: HashMap<(i32, i32), Task<(Chunk)>>,
    generator: TerrainGenerator
}

impl World {
    fn create_chunk(&mut self, position: (i32, i32), loading_pool: &Res<AsyncComputeTaskPool>,) {
        let loading_task: Task<Chunk> = Self::generate_chunk(position, TerrainGenerator::default(), loading_pool);
        self.loading_ledger.insert(position, loading_task);
    }

    fn generate_chunk(position: (i32, i32), generator: TerrainGenerator, loading_pool: &Res<AsyncComputeTaskPool>) -> Task<Chunk> {
        loading_pool.spawn(async move  {
            let mut chunk = Chunk::new(position);
            chunk.generate_voxels(&generator);
            chunk.generate_chunk_meshes();
            chunk
        })
    }

    fn render_chunk(&mut self, position: (i32, i32), loading_pool: &Res<AsyncComputeTaskPool>,) {
        if !self.loading_ledger.contains_key(&position) && !self.chunk_ledger.contains_key(&position) {
            self.create_chunk(position, loading_pool);
        }
    }

    fn load_chunks(&mut self, commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) {
        info!("{}", self.loading_ledger.len());
        let mut chunks: Vec<((i32, i32), Chunk)> = Vec::new();
        for (position, loading_task) in self.loading_ledger.iter_mut() {
            let result: Option<Chunk> = future::block_on(future::poll_once(loading_task));
            if result.is_some() {
                chunks.push((position.clone(), result.unwrap()));
            }
        }
        while !chunks.is_empty() {
            let (position, chunk) = chunks.remove(0);
            self.loading_ledger.remove(&position);
            self.chunk_ledger.insert(position.clone(), chunk);
            self.generate_bevy_chunk(position.clone(), commands, meshes, materials);
        }
    }

    fn generate_bevy_chunk(&mut self, position: (i32, i32), commands: &mut Commands, meshes: &mut ResMut<Assets<Mesh>>, materials: &mut ResMut<Assets<StandardMaterial>>) {
        let chunk = self.chunk_ledger.get_mut(&position).unwrap();
        if chunk.is_mesh_calculated() {
            let parent_chunk = commands.spawn().id();
            commands.entity(self.terrain_entity.unwrap()).add_child(parent_chunk);
            let sections = chunk.generate_bevy_meshes();
            for mesh in sections {
                let section = commands.spawn_bundle(PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.get_handle(&self.terrain_material),
                    ..default()
                }).id();
                commands.entity(parent_chunk).add_child(section);
            }
            self.bevy_chunk_ledger.insert(position, parent_chunk);
        }
        info!("Spawning chunk");
    }

    fn create_material(&mut self, materials: &mut ResMut<Assets<StandardMaterial>>) {
        self.terrain_material = materials.add(Color::GREEN.into());
    }
}

pub fn setup_world(
    mut commands: Commands,
    mut world: ResMut<World>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    world.terrain_entity = Some(commands.spawn().insert(Terrain).id());
    world.create_material(&mut materials);

    // directional 'sun' light
    const HALF_SIZE: f32 = 40.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            illuminance: 10000.,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
}

pub fn update_world(
    mut commands: Commands,
    mut world: ResMut<World>,
    mut terrain: Query<Entity, With<Terrain>>,
    mut loading_pool: Res<AsyncComputeTaskPool>,
    mut player_query: Query<&Transform, With<PlayerController>>
) {
    let terrain: Entity = terrain.single();
    let player: &Transform = player_query.single();
    let chunk_x = (player.translation.x / CHUNK_LENGTH as f32).floor() as i32;
    let chunk_z = (player.translation.z / CHUNK_LENGTH as f32).floor() as i32;

    for i in 0..16 {
        let min_x = chunk_x - i;
        let min_z = chunk_z - i;
        let max_x = chunk_x + i;
        let max_z = chunk_z + i;
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                world.render_chunk((x, z), &loading_pool);
            }
        }
    }
    //info!("X: {}, Z: {}", chunk_x, chunk_z);
    //info!("PX: {}, PZ: {}", player.translation.x, player.translation.z);
}

pub fn load_chunks(
    mut world: ResMut<World>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    world.load_chunks(&mut commands, &mut meshes, &mut materials);
}