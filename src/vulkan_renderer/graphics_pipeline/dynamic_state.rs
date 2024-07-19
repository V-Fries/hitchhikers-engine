use ash::vk;

#[derive(Default)]
pub struct DynamicStateCreateInfo<'a> {
    #[allow(dead_code)]
    dynamic_states: Box<[vk::DynamicState]>,
    create_info: vk::PipelineDynamicStateCreateInfo<'a>,
}

impl DynamicStateCreateInfo<'_> {
    pub fn new() -> Self {
        let dynamic_states = vec![
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
        ].into_boxed_slice();

        let create_info = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        Self { dynamic_states, create_info }
    }

    pub fn create_info(&self) -> &vk::PipelineDynamicStateCreateInfo {
        &self.create_info
    }
}
