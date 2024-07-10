use ash::vk;

pub fn input_assembly_state_create_info<'a>() -> vk::PipelineInputAssemblyStateCreateInfo<'a> {
    vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
}
