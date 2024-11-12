mod create_index_buffer;
mod create_uniform_buffers;
mod create_vertex_buffer;
mod descriptors;
mod errors;
mod image;

use std::{ffi::c_void, sync::LazyLock};

use crate::{
    utils::{Defer, Result, ScopeGuard},
    vertex::Vertex,
};

use super::{
    buffer::Buffer, render_targets::RenderTargets, vulkan_context::VulkanContext,
    vulkan_interface::VulkanInterface, NB_OF_FRAMES_IN_FLIGHT_USIZE,
};
use crate::error_struct;
use ash::vk;
use create_index_buffer::create_index_buffer;
use create_uniform_buffers::create_uniform_buffers;
use create_vertex_buffer::create_vertex_buffer;
use descriptors::create_descriptor_pool;
use descriptors::create_descriptor_sets;
use image::Image;
use image_parser::ppm::PpmFilePath;

error_struct!(
    FailedToFindMemoryTypeIndex,
    "Failed to find memory type index when trying to allocate memory for a buffer"
);

// TODO remove this
pub static VERTICES: LazyLock<[Vertex; 4]> = LazyLock::new(|| {
    [
        Vertex::new([-0.5, -0.5], [1.0, 0.0, 0.0]),
        Vertex::new([0.5, -0.5], [0.0, 1.0, 0.0]),
        Vertex::new([0.5, 0.5], [0.0, 0.0, 1.0]),
        Vertex::new([-0.5, 0.5], [1.0, 1.0, 1.0]),
    ]
});

// TODO remove this
pub static INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

// TODO remove this
pub const PPM_FILE_PATH: &str = "assets/textures/test.ppm";

pub struct Memory {
    vertex_buffer: Buffer,
    index_buffer: Buffer,

    uniform_buffers: [Buffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],
    mapped_uniform_buffers: [*mut c_void; NB_OF_FRAMES_IN_FLIGHT_USIZE],

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: [vk::DescriptorSet; NB_OF_FRAMES_IN_FLIGHT_USIZE],

    texture: Image,
}

impl Memory {
    pub unsafe fn new(
        context: &VulkanContext,
        interface: &VulkanInterface,
        render_targets: &RenderTargets,
    ) -> Result<Self> {
        let vertex_buffer = create_vertex_buffer(context, interface)?
            .defer(|mut vertex_buffer| vertex_buffer.destroy(context.device()));

        let index_buffer = create_index_buffer(context, interface)?
            .defer(|mut index_buffer| index_buffer.destroy(context.device()));

        let (uniform_buffers, mapped_uniform_buffers) = create_uniform_buffers(context)?;
        let uniform_buffers = uniform_buffers.defer(|mut uniform_buffers| {
            Self::destroy_uniform_buffers(context.device(), &mut uniform_buffers)
        });

        let descriptor_pool = create_descriptor_pool(context.device())?.defer(|descriptor_pool| {
            context
                .device()
                .destroy_descriptor_pool(descriptor_pool, None)
        });

        // Destroyed automatically when descriptor_pool is destroyed
        let descriptor_sets = create_descriptor_sets(
            context.device(),
            render_targets.descriptor_set_layout(),
            *descriptor_pool,
            &uniform_buffers,
        )?;

        let image = image_parser::Image::try_from(PpmFilePath(PPM_FILE_PATH))?;
        let texture = Image::from_texture_image(context, interface, &image)?
            .defer(|mut texture| texture.destroy(context.device()));

        Ok(Self {
            texture: ScopeGuard::into_inner(texture),
            descriptor_sets,
            descriptor_pool: ScopeGuard::into_inner(descriptor_pool),
            mapped_uniform_buffers,
            uniform_buffers: ScopeGuard::into_inner(uniform_buffers),
            index_buffer: ScopeGuard::into_inner(index_buffer),
            vertex_buffer: ScopeGuard::into_inner(vertex_buffer),
        })
    }

    pub fn find_memory_type_index(
        context: &VulkanContext,
        memory_type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32, FailedToFindMemoryTypeIndex> {
        unsafe {
            context
                .instance()
                .get_physical_device_memory_properties(context.physical_device())
        }
        .memory_types
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            memory_type_filter & (1 << index) != 0
                && memory_type.property_flags & properties == properties
        })
        .map(|(index, _)| index as u32)
        .ok_or(FailedToFindMemoryTypeIndex {})
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn mapped_uniform_buffers(&self) -> &[*mut c_void; NB_OF_FRAMES_IN_FLIGHT_USIZE] {
        &self.mapped_uniform_buffers
    }

    pub fn descriptor_sets(&self) -> &[vk::DescriptorSet; NB_OF_FRAMES_IN_FLIGHT_USIZE] {
        &self.descriptor_sets
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        device.destroy_descriptor_pool(self.descriptor_pool, None);
        Self::destroy_uniform_buffers(device, &mut self.uniform_buffers);
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
        self.texture.destroy(device);
    }

    pub unsafe fn destroy_uniform_buffers(device: &ash::Device, buffers: &mut [Buffer]) {
        for buffer in buffers {
            unsafe {
                device.unmap_memory(buffer.memory());
                buffer.destroy(device);
            }
        }
    }
}
