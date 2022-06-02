#[derive(Clone, Copy)]
pub struct Voxel {
    pub material: VoxelType
}

impl Voxel {
    pub fn air() -> Self {
        Self { material: VoxelType::Air }
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum VoxelType {
    Air,
    Grass,
    Dirt,
    Stone,
}