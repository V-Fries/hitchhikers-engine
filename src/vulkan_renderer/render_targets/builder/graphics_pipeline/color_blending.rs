use ash::vk;

pub struct ColorBlendStateCreateInfo<'a> {
    #[allow(dead_code)]
    color_blend_attachment_state: Box<[vk::PipelineColorBlendAttachmentState]>,
    create_info: vk::PipelineColorBlendStateCreateInfo<'a>,
}

impl ColorBlendStateCreateInfo<'_> {
    pub fn new() -> Self {
        let color_blend_attachment_state = Self::create_color_blend_attachment_state();
        let create_info = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: vk::FALSE,
            attachment_count: color_blend_attachment_state.len() as u32,
            p_attachments: color_blend_attachment_state.as_ptr(),
            ..Default::default()
        };
        Self { color_blend_attachment_state, create_info }
    }

    pub fn create_info(&self) -> &vk::PipelineColorBlendStateCreateInfo {
        &self.create_info
    }

    fn create_color_blend_attachment_state() -> Box<[vk::PipelineColorBlendAttachmentState]> {
        vec![
            vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(
                    vk::ColorComponentFlags::R
                        | vk::ColorComponentFlags::G
                        | vk::ColorComponentFlags::B
                        | vk::ColorComponentFlags::A
                )
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ZERO)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD)
        ].into_boxed_slice()
    }
}
