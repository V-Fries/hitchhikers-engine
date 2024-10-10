use ash::prelude::VkResult;
use ash::vk;

pub fn create_pipeline_layout(device: &ash::Device) -> VkResult<vk::PipelineLayout> {
    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::default();
    unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None) }
}
