use super::super::errors::FailedToCreatePipeline;
use super::color_blending::ColorBlendStateCreateInfo;
use super::depth_stencil_state_create_info::depth_stencil_state_create_info;
use super::dynamic_state::DynamicStateCreateInfo;
use super::input_assembly::input_assembly_state_create_info;
use super::multisampling::multisample_state_create_info;
use super::pipeline_layout::create_pipeline_layout;
use super::rasterizer::rasterizer_state_create_info;
use super::shader::ShaderStageCreateInfos;
use super::vertex_input::vertex_input_state_create_info;
use super::viewport::ViewportStateCreateInfo;
use crate::vertex::Vertex;
use ash::vk;
use rs42::Result;

pub unsafe fn create_graphics_pipeline(
    device: &ash::Device,
    swapchain_extent: &vk::Extent2D,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
    let shader_stage_create_infos = ShaderStageCreateInfos::new(device)?;
    let binding_descriptions = [Vertex::get_binding_description()];
    let attributes_description = Vertex::get_attributes_descriptions();
    let vertex_input_state_create_info =
        vertex_input_state_create_info(&binding_descriptions, &attributes_description);
    let input_assembly_state_create_info = input_assembly_state_create_info();
    let viewport_state_create_info = ViewportStateCreateInfo::new(swapchain_extent);
    let rasterizer_state_create_info = rasterizer_state_create_info();
    let multisample_state_create_info = multisample_state_create_info();
    let color_blend_state_create_info = ColorBlendStateCreateInfo::new();
    let dynamic_state_create_info = DynamicStateCreateInfo::new();
    let depth_stencil_state_create_info = depth_stencil_state_create_info();
    let pipeline_layout = create_pipeline_layout(device, descriptor_set_layout)?;

    let create_infos = [vk::GraphicsPipelineCreateInfo::default()
        .stages(shader_stage_create_infos.create_infos())
        .vertex_input_state(&vertex_input_state_create_info)
        .input_assembly_state(&input_assembly_state_create_info)
        .viewport_state(viewport_state_create_info.create_info())
        .rasterization_state(&rasterizer_state_create_info)
        .multisample_state(&multisample_state_create_info)
        .color_blend_state(color_blend_state_create_info.create_info())
        .dynamic_state(dynamic_state_create_info.create_info())
        .depth_stencil_state(&depth_stencil_state_create_info)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)];

    let graphics_pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &create_infos, None)
            .map_err(|err| {
                device.destroy_pipeline_layout(pipeline_layout, None);
                FailedToCreatePipeline::new(err)
            })?[0]
    };

    Ok((pipeline_layout, graphics_pipeline))
}
