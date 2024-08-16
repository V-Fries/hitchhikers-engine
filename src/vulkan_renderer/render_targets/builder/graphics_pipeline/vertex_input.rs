use ash::vk;

pub fn vertex_input_state_create_info<'a>() -> vk::PipelineVertexInputStateCreateInfo<'a> {
    vk::PipelineVertexInputStateCreateInfo::default()
}
