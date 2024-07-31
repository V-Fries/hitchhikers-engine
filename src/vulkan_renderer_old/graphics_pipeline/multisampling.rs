use ash::vk;

pub fn multisample_state_create_info<'a>() -> vk::PipelineMultisampleStateCreateInfo<'a> {
    vk::PipelineMultisampleStateCreateInfo::default()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .min_sample_shading(1.)
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false)
}
