use std::cmp::max;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use crate::game::world::generator::TerrainGenerator;
use crate::game::world::voxel::*;

pub const CHUNK_LENGTH: usize = 16;
pub const CHUNK_AREA: usize = CHUNK_LENGTH * CHUNK_LENGTH;
pub const CHUNK_VOLUME: usize = CHUNK_LENGTH * CHUNK_LENGTH * CHUNK_LENGTH;

pub struct Chunk {
    position: (i32, i32),
    sections: Vec<ChunkSection>,
    terrain_generated: bool,
    mesh_calculated: bool
}

impl Chunk {
    pub(crate) fn new(position: (i32, i32)) -> Self {
        Self {
            position,
            sections: Vec::new(),
            terrain_generated: false,
            mesh_calculated: false
        }
    }

    pub fn generate_voxels(&mut self, generator: &TerrainGenerator) {
        for x in 0..CHUNK_LENGTH {
            for z in 0..CHUNK_LENGTH {
                let height = generator.get_height(x as i32 + self.position.0 * CHUNK_LENGTH as i32, z as i32 + self.position.1 * CHUNK_LENGTH as i32) as u32;
                self.set_voxel((x as u32, height, z as u32), VoxelType::Grass);
                for y in (max(height-3, 0))..height {
                    self.set_voxel((x as u32, y, z as u32), VoxelType::Dirt)
                }
            }
        }
        self.terrain_generated = true;
    }

    pub fn is_generated(&self) -> bool {
        self.terrain_generated
    }

    pub fn is_mesh_calculated(&self) -> bool {
        self.mesh_calculated
    }

    pub fn get_position(&self) -> (i32, i32) {
        self.position
    }

    pub fn generate_bevy_meshes(&self) -> Vec<Mesh> {
        if !self.terrain_generated { error!("Chunk not loaded yet") }
        self.sections.iter().map(|section| -> Mesh {
            section.create_bevy_mesh()
        }).collect()
    }

    pub fn generate_chunk_meshes(&mut self) {
        if !self.terrain_generated { error!("Chunk not loaded yet") }
        for mut section in &mut self.sections {
            if !section.mesh_generated {
                section.generate_chunk_mesh();
            }
        }

        self.mesh_calculated = true;
    }

    pub fn set_voxel(&mut self, position: (u32, u32, u32), material: VoxelType) {
        if !Self::in_bounds(position) { error!("Voxel position out of bounds"); }
        self.add_y_sections(position.1);
        let section: &mut ChunkSection = &mut self.sections[position.1 as usize / CHUNK_LENGTH];
        section.set_voxel((position.0, position.1 % CHUNK_LENGTH as u32, position.2), material);
    }

    fn add_y_sections(&mut self, y: u32) {
        let index = y / CHUNK_LENGTH as u32;
        while self.sections.len() <= index as usize {
            self.sections.push(ChunkSection::new((self.position.0, self.sections.len() as i32, self.position.1)))
        }
    }

    fn in_bounds(position: (u32, u32, u32)) -> bool {
        if position.0 >= CHUNK_LENGTH as u32 { return false; }
        if position.2 >= CHUNK_LENGTH as u32 { return false; }
        return true;
    }
}

pub struct ChunkSection {
    voxels: Box<[Voxel; CHUNK_VOLUME]>,
    position: (i32, i32, i32),
    chunk_mesh: ChunkMesh,
    mesh_generated: bool,
}

#[derive(Default)]
struct Directions {
    front: (i32, i32, i32),
    back: (i32, i32, i32),
    left: (i32, i32, i32),
    right: (i32, i32, i32),
    top: (i32, i32, i32),
    bottom: (i32, i32, i32),
}

impl Directions {
    fn new() -> Self {
        Self {
            ..default()
        }
    }
    fn update(&mut self, position: (i32, i32, i32)) {
        self.front = (position.0, position.1, position.2 + 1);
        self.back = (position.0, position.1, position.2 - 1);
        self.left = (position.0 - 1, position.1, position.2);
        self.right = (position.0 + 1, position.1, position.2);
        self.top = (position.0, position.1 + 1, position.2);
        self.bottom = (position.0, position.1 - 1, position.2);
    }
}

impl ChunkSection {
    fn new(position: (i32, i32, i32)) -> Self {
        Self {
            voxels: Box::new([Voxel::air(); CHUNK_VOLUME]),
            position,
            chunk_mesh: ChunkMesh::new(),
            mesh_generated: false
        }
    }

    fn generate_chunk_mesh(&mut self) {
        self.mesh_generated = true;

        let mut builder = ChunkBuilder::new(self);
        builder.build();
    }

    fn create_bevy_mesh(&self) -> Mesh {
        if !self.mesh_generated {
            error!("Mesh data not generated yet")
        }
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.chunk_mesh.vertices.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.chunk_mesh.uvs.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.chunk_mesh.normals.clone());
        mesh.set_indices(Some(Indices::U32(self.chunk_mesh.indices.clone())));
        return mesh;
    }

    pub fn set_voxel(&mut self, position: (u32, u32, u32), material: VoxelType) {
        if !Self::in_bounds(position.0, position.1, position.2) { error!("Attempting to set voxel out of bounds") }
        self.voxels[Self::get_index(position.0, position.1, position.2)].material = material;
    }

    pub fn get_voxel(&mut self, position: (u32, u32, u32)) -> &Voxel {
        if !Self::in_bounds(position.0, position.1, position.2) { error!("Attempting to get voxel out of bounds") }
        &self.voxels[Self::get_index(position.0, position.1, position.2)]
    }

    pub fn in_bounds(x: u32, y: u32, z: u32) -> bool {
        if x >= CHUNK_LENGTH as u32 { return false; }
        if y >= CHUNK_LENGTH as u32 { return false; }
        if z >= CHUNK_LENGTH as u32 { return false; }
        return true;
    }

    fn get_index(x: u32, y: u32, z: u32) -> usize {
        (y * CHUNK_AREA as u32 + z * CHUNK_LENGTH as u32 + x) as usize
    }
}

struct ChunkMesh {
    vertices: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>
}

impl ChunkMesh {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new()
        }
    }
}

struct ChunkBuilder<'a> {
    chunk: &'a mut ChunkSection,
    index_count: u32
}

impl<'a> ChunkBuilder<'a> {
    fn new(chunk: &'a mut ChunkSection) -> Self {
        Self {
            chunk,
            index_count: 0
        }
    }

    fn build(&mut self) {
        self.chunk.chunk_mesh.vertices.clear();
        self.chunk.chunk_mesh.indices.clear();
        self.index_count = 0;

        let mut directions = Directions::new();

        for i in 0..CHUNK_VOLUME {
            let voxel = &self.chunk.voxels[i];
            if voxel.material == VoxelType::Air {
                continue;
            }

            let x = (i % CHUNK_LENGTH) as u32;
            let y = (i / CHUNK_AREA) as u32;
            let z = (i / CHUNK_LENGTH % CHUNK_LENGTH) as u32;

            directions.update((x as i32, y as i32, z as i32));

            self.try_add_face(&FRONT_FACE, &FRONT_FACE_UVS, (0., 0., 1.), (x, y, z), directions.front);
            self.try_add_face(&BACK_FACE, &BACK_FACE_UVS, (0., 0., -1.), (x, y, z), directions.back);

            self.try_add_face(&RIGHT_FACE, &FRONT_FACE_UVS, (1., 0., 0.), (x, y, z), directions.right);
            self.try_add_face(&LEFT_FACE, &BACK_FACE_UVS, (-1., 0., 0.), (x, y, z), directions.left);

            self.try_add_face(&TOP_FACE, &FRONT_FACE_UVS, (0., 1., 0.), (x, y, z), directions.top);
            self.try_add_face(&BOTTOM_FACE, &BACK_FACE_UVS, (0., -1., 0.), (x, y, z), directions.bottom);
        }
    }

    fn try_add_face(&mut self, vertices: &[[f32; 3]; 4], uvs: &[[f32; 2]; 4], normal: (f32, f32, f32), block_position: (u32, u32, u32), adjacent_position: (i32, i32, i32)) {
        if adjacent_position.0 > 0 &&
            adjacent_position.1 > 0 &&
            adjacent_position.2 > 0 &&
            ChunkSection::in_bounds(adjacent_position.0 as u32, adjacent_position.1 as u32, adjacent_position.2 as u32) {
            if self.chunk.get_voxel((adjacent_position.0 as u32, adjacent_position.1 as u32, adjacent_position.2 as u32)).material != VoxelType::Air {
                return;
            }
        }

        for i in 0..4 {
            let vertex = vertices[i];
            self.chunk.chunk_mesh.vertices.push([
                vertex[0] + (block_position.0 as i32 + self.chunk.position.0 * CHUNK_LENGTH as i32) as f32,
                vertex[1] + (block_position.1 as i32 + self.chunk.position.1 * CHUNK_LENGTH as i32) as f32,
                vertex[2] + (block_position.2 as i32 + self.chunk.position.2 * CHUNK_LENGTH as i32) as f32,
            ]);
            self.chunk.chunk_mesh.uvs.push(uvs[i]);
            self.chunk.chunk_mesh.normals.push([normal.0, normal.1, normal.2]);
        }

        self.chunk.chunk_mesh.indices.extend([
            self.index_count, self.index_count + 1, self.index_count + 2,
            self.index_count + 2, self.index_count + 3, self.index_count
        ]);

        self.index_count += 4;
    }
}

const FRONT_FACE: [[f32; 3]; 4] = [
    [0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]
];
const FRONT_FACE_UVS: [[f32; 2]; 4] = [
    [0., 0.], [1., 0.], [1., 1.], [1., 1.]
];
const BACK_FACE: [[f32; 3]; 4] = [
    [1., 0., 0.], [0., 0., 0.], [0., 1., 0.], [1., 1., 0.]
];
const BACK_FACE_UVS: [[f32; 2]; 4] = [
    [0., 0.], [1., 0.], [1., 1.], [1., 1.]
];

const LEFT_FACE: [[f32; 3]; 4] = [
    [0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]
];
const LEFT_FACE_UVS: [[f32; 2]; 4] = [
    [0., 0.], [1., 0.], [1., 1.], [1., 1.]
];
const RIGHT_FACE: [[f32; 3]; 4] = [
    [1., 0., 1.], [1., 0., 0.], [1., 1., 0.], [1., 1., 1.]
];
const RIGHT_FACE_UVS: [[f32; 2]; 4] = [
    [0., 0.], [1., 0.], [1., 1.], [1., 1.]
];

const TOP_FACE: [[f32; 3]; 4] = [
    [0., 1., 1.], [1., 1., 1.], [1., 1., 0.], [0., 1., 0.]
];
const TOP_FACE_UVS: [[f32; 2]; 4] = [
    [0., 0.], [1., 0.], [1., 1.], [1., 1.]
];
const BOTTOM_FACE: [[f32; 3]; 4] = [
    [0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]
];
const BOTTOM_FACE_UVS: [[f32; 2]; 4] = [
    [0., 0.], [1., 0.], [1., 1.], [1., 1.]
];

