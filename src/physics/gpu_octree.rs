use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuOctreeNode {
    pub center_of_mass: [f32; 4],
    pub center: [f32; 4], // xyz + half_size
    pub mass: f32,
    pub is_leaf: u32,
    pub body_index: u32,
    pub _padding: u32,
    pub children: [u32; 8],
}
