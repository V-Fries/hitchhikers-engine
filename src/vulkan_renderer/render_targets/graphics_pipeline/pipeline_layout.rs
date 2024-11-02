use ash::prelude::VkResult;
use ash::vk;

pub fn create_pipeline_layout(
    device: &ash::Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> VkResult<vk::PipelineLayout> {
    unsafe {
        device.create_pipeline_layout(
            &vk::PipelineLayoutCreateInfo::default().set_layouts(&[descriptor_set_layout]),
            None,
        )
    }
}
