use ash::prelude::VkResult;
use ash::vk;

pub unsafe fn create_image_views(
    device: &ash::Device,
    swapchain_images: &[vk::Image],
    format: vk::Format,
) -> VkResult<Box<[vk::ImageView]>> {
    let mut image_views = Vec::with_capacity(swapchain_images.len());
    let create_info = get_image_view_create_info(format);

    for swapchain_image in swapchain_images {
        let image_view =
            unsafe { device.create_image_view(&create_info.image(*swapchain_image), None) }
                .inspect_err(|_| {
                    for image_view in image_views.iter() {
                        unsafe { device.destroy_image_view(*image_view, None) }
                    }
                })?;

        image_views.push(image_view);
    }

    Ok(image_views.into_boxed_slice())
}

fn get_image_view_create_info(format: vk::Format) -> vk::ImageViewCreateInfo<'static> {
    vk::ImageViewCreateInfo::default()
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .components(
            vk::ComponentMapping::default()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY),
        )
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        )
}
