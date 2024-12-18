use std::mem::offset_of;

use ash::vk;
use linear_algebra::Vector;

pub type Vec2 = Vector<f32, 2>;
pub type Vec3 = Vector<f32, 3>;

pub type Color = Vec3;

// TODO research doing different buffers for positions and texture coordinates
#[derive(Clone)]
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    color: Color, // TODO Might remove this
    texture_coordinate: Vec2,
}

impl Vertex {
    fn into_tuple_of_bits(self) -> ([u32; 3], [u32; 3], [u32; 2]) {
        (
            self.position.into_scalars().map(|e| e.to_bits()),
            self.color.into_scalars().map(|e| e.to_bits()),
            self.texture_coordinate.into_scalars().map(|e| e.to_bits()),
        )
    }
}

impl std::hash::Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.clone().into_tuple_of_bits().hash(state)
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.clone()
            .into_tuple_of_bits()
            .eq(&other.clone().into_tuple_of_bits())
    }
}

impl Eq for Vertex {}

impl Vertex {
    pub fn new(
        position: impl Into<Vec3>,
        color: impl Into<Color>,
        texture_coordinate: impl Into<Vec2>,
    ) -> Self {
        Self {
            position: position.into(),
            color: color.into(),
            texture_coordinate: texture_coordinate.into(),
        }
    }

    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn get_attributes_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, position) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, color) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, texture_coordinate) as u32),
        ]
    }
}
