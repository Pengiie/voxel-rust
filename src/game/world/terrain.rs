use crate::PbrBundle;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::Mesh;
use bevy::render::mesh::{MeshVertexAttribute, PrimitiveTopology};

#[derive(Bundle)]
struct TerrainBundle {
    #[bundle]
    model: PbrBundle
}