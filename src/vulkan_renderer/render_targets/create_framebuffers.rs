use ash::vk;

use rs42::Result;

pub unsafe fn create_framebuffers(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: &[vk::ImageView],
    depth_buffer_image_view: vk::ImageView,
    color_buffer_image_view: vk::ImageView,
) -> Result<Box<[vk::Framebuffer]>> {
    let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());

    for image_view in swapchain_image_views {
        let attachments = [
            color_buffer_image_view,
            depth_buffer_image_view,
            *image_view,
        ];
        let create_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(swapchain_extent.width)
            .height(swapchain_extent.height)
            .layers(1);
        let framebuffer = unsafe {
            device
                .create_framebuffer(&create_info, None)
                .inspect_err(|_| {
                    for framebuffer in framebuffers.iter() {
                        device.destroy_framebuffer(*framebuffer, None);
                    }
                })?
        };
        framebuffers.push(framebuffer);
    }
    Ok(framebuffers.into_boxed_slice())
}
