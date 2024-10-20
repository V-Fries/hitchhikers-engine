use ash::vk;

use crate::utils::Result;

pub unsafe fn create_framebuffers(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: &[vk::ImageView],
) -> Result<Box<[vk::Framebuffer]>> {
    let mut framebuffers = Vec::with_capacity(swapchain_image_views.len());

    for image in swapchain_image_views {
        let attachments = [*image];
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
