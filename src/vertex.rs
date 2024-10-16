use std::mem::offset_of;

use ash::vk;
use linear_algebra::Vector;

pub type Vec2 = Vector<f32, 2>;

pub type Color = Vector<f32, 3>;

#[repr(C)]
pub struct Vertex {
    pos: Vec2,
    color: Color,
}

impl Vertex {
    pub fn new(pos: impl Into<Vec2>, color: impl Into<Color>) -> Self {
        Self {
            pos: pos.into(),
            color: color.into(),
        }
    }

    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn get_attributes_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, color) as u32),
        ]
    }
}
