mod create_index_buffer;
mod create_uniform_buffers;
mod create_vertex_buffer;
mod descriptors;
mod errors;
mod image;

use std::ffi::c_void;

use model::{Model, ObjFile};
use rs42::{
    scope_guard::{Defer, ScopeGuard},
    Result,
};

use super::{
    buffer::Buffer, render_targets::RenderTargets, vulkan_context::VulkanContext,
    vulkan_interface::VulkanInterface, NB_OF_FRAMES_IN_FLIGHT_USIZE,
};
use ash::{prelude::VkResult, vk};
use create_index_buffer::create_index_buffer;
use create_uniform_buffers::create_uniform_buffers;
use create_vertex_buffer::create_vertex_buffer;
use descriptors::create_descriptor_pool;
use descriptors::create_descriptor_sets;
pub use image::{Image, ImageCreateInfo};
use image_parser::ppm::PpmFilePath;
use rs42::error_struct_custom_display;

error_struct_custom_display!(
    FailedToFindMemoryTypeIndex,
    "Failed to find memory type index when trying to allocate memory for a buffer"
);

// TODO remove this
pub const PPM_FILE_PATH: &str = "assets/textures/viking_room.ppm";
pub const OBJ_FILE_PATH: &str = "assets/obj/viking_room.obj";

pub struct Memory {
    is_destroyed: bool,

    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_buffer_len: u32,

    uniform_buffers: [Buffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],
    mapped_uniform_buffers: [*mut c_void; NB_OF_FRAMES_IN_FLIGHT_USIZE],

    descriptors_are_destroyed: bool,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: [vk::DescriptorSet; NB_OF_FRAMES_IN_FLIGHT_USIZE],

    texture: Image,
    sampler: vk::Sampler,
}

impl Memory {
    pub unsafe fn new(
        context: &VulkanContext,
        interface: &VulkanInterface,
        render_targets: &RenderTargets,
    ) -> Result<Self> {
        let model = Model::try_from(ObjFile(OBJ_FILE_PATH))?;
        let vertex_buffer = create_vertex_buffer(context, interface, model.vertices())?
            .defer(|mut vertex_buffer| vertex_buffer.destroy(context.device()));

        let index_buffer = create_index_buffer(context, interface, model.vertex_indices())?
            .defer(|mut index_buffer| index_buffer.destroy(context.device()));

        let (uniform_buffers, mapped_uniform_buffers) = create_uniform_buffers(context)?;
        let uniform_buffers = uniform_buffers.defer(|mut uniform_buffers| {
            Self::destroy_uniform_buffers(context.device(), &mut uniform_buffers)
        });

        let image = image_parser::Image::try_from(PpmFilePath(PPM_FILE_PATH))?;
        let texture = Image::from_texture_image(context, interface, &image)?
            .defer(|mut texture| texture.destroy(context.device()));

        let sampler = Self::init_sampler(context)?
            .defer(|sampler| unsafe { context.device().destroy_sampler(sampler, None) });

        let (descriptor_pool, descriptor_sets) = Self::create_descriptors(
            context,
            render_targets,
            &uniform_buffers,
            texture.image_view(),
            *sampler,
        )?;
        let descriptor_pool = descriptor_pool.defer(|descriptor_pool| {
            context
                .device()
                .destroy_descriptor_pool(descriptor_pool, None)
        });

        Ok(Self {
            sampler: ScopeGuard::into_inner(sampler),
            texture: ScopeGuard::into_inner(texture),
            descriptor_sets,
            descriptor_pool: ScopeGuard::into_inner(descriptor_pool),
            descriptors_are_destroyed: false,
            mapped_uniform_buffers,
            uniform_buffers: ScopeGuard::into_inner(uniform_buffers),
            index_buffer: ScopeGuard::into_inner(index_buffer),
            index_buffer_len: model.vertex_indices().len() as u32,
            vertex_buffer: ScopeGuard::into_inner(vertex_buffer),
            is_destroyed: false,
        })
    }

    pub unsafe fn create_descriptors(
        context: &VulkanContext,
        render_targets: &RenderTargets,
        uniform_buffers: &[Buffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],
        texture_image_view: vk::ImageView,
        texture_sampler: vk::Sampler,
    ) -> Result<(
        vk::DescriptorPool,
        [vk::DescriptorSet; NB_OF_FRAMES_IN_FLIGHT_USIZE],
    )> {
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
            uniform_buffers,
            texture_image_view,
            texture_sampler,
        )?;

        Ok((ScopeGuard::into_inner(descriptor_pool), descriptor_sets))
    }

    pub unsafe fn set_descriptors(
        &mut self,
        pool: vk::DescriptorPool,
        sets: [vk::DescriptorSet; NB_OF_FRAMES_IN_FLIGHT_USIZE],
    ) {
        debug_assert!(!self.is_destroyed);
        debug_assert!(self.descriptors_are_destroyed);

        self.descriptor_pool = pool;
        self.descriptor_sets = sets;
        self.descriptors_are_destroyed = false;
    }

    pub unsafe fn destroy_descriptors(&mut self, device: &ash::Device) {
        debug_assert!(!self.is_destroyed);
        debug_assert!(!self.descriptors_are_destroyed);

        device.destroy_descriptor_pool(self.descriptor_pool, None);
        self.descriptors_are_destroyed = true;
    }

    fn init_sampler(context: &VulkanContext) -> VkResult<vk::Sampler> {
        let (anisotropy_enable, max_anisotropy) = Self::get_anisotropy_settings(context);

        unsafe {
            context.device().create_sampler(
                &vk::SamplerCreateInfo::default()
                    .mag_filter(vk::Filter::LINEAR)
                    .min_filter(vk::Filter::LINEAR)
                    .address_mode_u(vk::SamplerAddressMode::REPEAT)
                    .address_mode_v(vk::SamplerAddressMode::REPEAT)
                    .address_mode_w(vk::SamplerAddressMode::REPEAT)
                    .anisotropy_enable(anisotropy_enable)
                    .max_anisotropy(max_anisotropy)
                    .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                    .unnormalized_coordinates(false)
                    .compare_enable(false)
                    .compare_op(vk::CompareOp::ALWAYS)
                    .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                    .mip_lod_bias(0.)
                    .min_lod(0.)
                    .max_lod(vk::LOD_CLAMP_NONE),
                None,
            )
        }
    }

    fn get_anisotropy_settings(context: &VulkanContext) -> (bool, f32) {
        if context.physical_device_features().sampler_anisotropy != 0 {
            (
                true,
                context
                    .physical_device_properties()
                    .limits
                    .max_sampler_anisotropy,
            )
        } else {
            (false, 1.)
        }
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
        debug_assert!(!self.is_destroyed);

        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        debug_assert!(!self.is_destroyed);

        &self.index_buffer
    }

    pub fn index_buffer_len(&self) -> u32 {
        debug_assert!(!self.is_destroyed);
        self.index_buffer_len
    }

    pub fn uniform_buffers(&self) -> &[Buffer; NB_OF_FRAMES_IN_FLIGHT_USIZE] {
        debug_assert!(!self.is_destroyed);

        &self.uniform_buffers
    }

    pub fn mapped_uniform_buffers(&self) -> &[*mut c_void; NB_OF_FRAMES_IN_FLIGHT_USIZE] {
        debug_assert!(!self.is_destroyed);

        &self.mapped_uniform_buffers
    }

    pub fn texture(&self) -> &Image {
        debug_assert!(!self.is_destroyed);

        &self.texture
    }

    pub fn sampler(&self) -> vk::Sampler {
        debug_assert!(!self.is_destroyed);

        self.sampler
    }

    pub fn descriptor_sets(&self) -> &[vk::DescriptorSet; NB_OF_FRAMES_IN_FLIGHT_USIZE] {
        debug_assert!(!self.is_destroyed);
        debug_assert!(!self.descriptors_are_destroyed);

        &self.descriptor_sets
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        // If an error occurs during swapchain recreation this function might be called twice
        if self.is_destroyed {
            return;
        }
        self.is_destroyed = true;

        if !self.descriptors_are_destroyed {
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.descriptors_are_destroyed = true;
        }
        Self::destroy_uniform_buffers(device, &mut self.uniform_buffers);
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
        self.texture.destroy(device);
        device.destroy_sampler(self.sampler, None);
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
