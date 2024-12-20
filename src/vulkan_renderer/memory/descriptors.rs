use ash::{prelude::VkResult, vk};

use crate::vulkan_renderer::buffer::Buffer;
use crate::vulkan_renderer::uniform_buffer_object::UniformBufferObject;
use crate::vulkan_renderer::{NB_OF_FRAMES_IN_FLIGHT, NB_OF_FRAMES_IN_FLIGHT_USIZE};
use rs42::{extensions::PipeLine, Result};

use super::errors::FailedToConvertDescriptorSetsVecToArray;

pub fn create_descriptor_pool(device: &ash::Device) -> VkResult<vk::DescriptorPool> {
    let pool_sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(NB_OF_FRAMES_IN_FLIGHT),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(NB_OF_FRAMES_IN_FLIGHT),
    ];

    unsafe {
        device.create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                .max_sets(NB_OF_FRAMES_IN_FLIGHT),
            None,
        )
    }
}

pub unsafe fn create_descriptor_sets(
    device: &ash::Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    uniform_buffers: &[Buffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,
) -> Result<[vk::DescriptorSet; NB_OF_FRAMES_IN_FLIGHT_USIZE]> {
    let layouts = [descriptor_set_layout; NB_OF_FRAMES_IN_FLIGHT_USIZE];

    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    let descriptor_sets: [vk::DescriptorSet; NB_OF_FRAMES_IN_FLIGHT_USIZE] = device
        .allocate_descriptor_sets(&allocate_info)?
        .try_into()
        .map_err(|vec: Vec<vk::DescriptorSet>| {
            FailedToConvertDescriptorSetsVecToArray::new(vec.len(), NB_OF_FRAMES_IN_FLIGHT_USIZE)
                .pipe(Box::new)
        })?;

    let buffer_infos = std::array::from_fn::<_, NB_OF_FRAMES_IN_FLIGHT_USIZE, _>(|i| {
        [vk::DescriptorBufferInfo::default()
            .buffer(uniform_buffers[i].buffer())
            .offset(0)
            .range(size_of::<UniformBufferObject>() as vk::DeviceSize)]
    });

    let image_infos = std::array::from_fn::<_, NB_OF_FRAMES_IN_FLIGHT_USIZE, _>(|_| {
        [vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture_image_view)
            .sampler(texture_sampler)]
    });

    let descriptor_writes = (0..NB_OF_FRAMES_IN_FLIGHT_USIZE)
        .flat_map(|i| {
            [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i])
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .buffer_info(&buffer_infos[i]),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i])
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .image_info(&image_infos[i]),
            ]
        })
        .collect::<Vec<vk::WriteDescriptorSet>>();

    device.update_descriptor_sets(&descriptor_writes, &[]);

    Ok(descriptor_sets)
}
