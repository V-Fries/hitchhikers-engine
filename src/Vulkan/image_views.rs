use ash::prelude::VkResult;
use ash::vk;

pub fn create_image_views(device: &ash::Device,
                          swap_chain_images: &[vk::Image],
                          format: vk::Format)
                          -> VkResult<Vec<vk::ImageView>> {
    swap_chain_images
        .iter()
        .map(|image| {
            let create_info = get_image_view_create_info(*image, format);
            unsafe { device.create_image_view(&create_info, None) }
        })
        .collect::<VkResult<Vec<_>>>()
}

fn get_image_view_create_info(image: vk::Image,
                              format: vk::Format)
                              -> vk::ImageViewCreateInfo<'static> {
    vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .components(vk::ComponentMapping::default()
            .r(vk::ComponentSwizzle::IDENTITY)
            .g(vk::ComponentSwizzle::IDENTITY)
            .b(vk::ComponentSwizzle::IDENTITY)
            .a(vk::ComponentSwizzle::IDENTITY)
        )
        .subresource_range(vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
        )
}
