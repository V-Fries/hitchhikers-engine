use ash::vk;

pub struct ViewportStateCreateInfo<'a> {
    #[allow(dead_code)]
    viewports: Box<[vk::Viewport]>,
    #[allow(dead_code)]
    scissors: Box<[vk::Rect2D]>,
    create_info: vk::PipelineViewportStateCreateInfo<'a>,
}

impl ViewportStateCreateInfo<'_> {
    pub fn new(swapchain_extent: &vk::Extent2D) -> Self {
        let viewports = viewports(swapchain_extent);
        let scissors = scissors(swapchain_extent);

        let create_info = vk::PipelineViewportStateCreateInfo {
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
            ..Default::default()
        };

        Self { viewports, scissors, create_info }
    }

    pub fn create_info(&self) -> &vk::PipelineViewportStateCreateInfo {
        &self.create_info
    }
}

fn viewports(swapchain_extent: &vk::Extent2D) -> Box<[vk::Viewport]> {
    vec![
        vk::Viewport::default()
            .x(0.)
            .y(0.)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.)
            .max_depth(1.)
    ].into_boxed_slice()
}

fn scissors(swapchain_extent: &vk::Extent2D) -> Box<[vk::Rect2D]> {
    vec![
        vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(*swapchain_extent)
    ].into_boxed_slice()
}
