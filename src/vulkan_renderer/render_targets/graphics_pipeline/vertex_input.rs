use ash::vk;

pub fn vertex_input_state_create_info<'a>(
    binding_descriptions: &'a [vk::VertexInputBindingDescription],
    attributes_description: &'a [vk::VertexInputAttributeDescription],
) -> vk::PipelineVertexInputStateCreateInfo<'a> {
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(attributes_description)
}
