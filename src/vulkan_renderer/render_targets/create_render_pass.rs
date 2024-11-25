use crate::{utils::Result, vulkan_renderer::vulkan_context::VulkanContext};
use ash::vk;

use super::create_depth_buffer::find_depth_buffer_format;

const DEPTH_BUFFER_LAYOUT: vk::ImageLayout = vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL;

pub unsafe fn create_render_pass(
    context: &VulkanContext,
    swapchain_format: vk::Format,
) -> Result<vk::RenderPass> {
    let attachment_descriptions = get_attachment_descriptions(context, swapchain_format)?;

    let color_attachment_references = [vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
    let depth_attachment_reference = vk::AttachmentReference::default()
        .attachment(1)
        .layout(DEPTH_BUFFER_LAYOUT);

    let subpass = [vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachment_references)
        .depth_stencil_attachment(&depth_attachment_reference)];

    let dependencies = get_dependencies();

    let render_pass_create_info = vk::RenderPassCreateInfo::default()
        .attachments(&attachment_descriptions)
        .subpasses(&subpass)
        .dependencies(&dependencies);

    Ok(unsafe {
        context
            .device()
            .create_render_pass(&render_pass_create_info, None)?
    })
}

fn get_attachment_descriptions(
    context: &VulkanContext,
    swapchain_format: vk::Format,
) -> Result<[vk::AttachmentDescription; 2]> {
    let color_attachment = vk::AttachmentDescription::default()
        .format(swapchain_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let depth_attachment = vk::AttachmentDescription::default()
        .format(find_depth_buffer_format(context)?)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(DEPTH_BUFFER_LAYOUT);

    Ok([color_attachment, depth_attachment])
}

fn get_dependencies() -> [vk::SubpassDependency; 1] {
    [vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
        )]
}
