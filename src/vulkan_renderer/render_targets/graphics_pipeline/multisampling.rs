use ash::vk;

pub fn multisample_state_create_info<'a>(
    sample_count: vk::SampleCountFlags,
) -> vk::PipelineMultisampleStateCreateInfo<'a> {
    vk::PipelineMultisampleStateCreateInfo::default()
        .sample_shading_enable(false)
        .rasterization_samples(sample_count)
        .min_sample_shading(1.)
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false)
}
