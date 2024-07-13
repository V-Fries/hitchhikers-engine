mod errors;

use std::ffi::CStr;
use ash::prelude::VkResult;
use ash::vk;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use crate::utils::Result;
use crate::const_str_to_cstr;
use crate::engine::errors::{FailedToCreateWindow, FailedToInitVulkan};
use crate::vulkan::Vulkan;

pub const ENGINE_NAME: &str = "HitchHiker's Engine";
pub const ENGINE_NAME_CSTR: &CStr = const_str_to_cstr!(ENGINE_NAME);

pub const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

pub struct Engine {
    vulkan: Vulkan,
    window: Window,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let window_attributes = Window::default_attributes()
            .with_title(ENGINE_NAME);

        let window = event_loop.create_window(window_attributes)
            .map_err(FailedToCreateWindow::new)?;

        Ok(Self {
            vulkan: Vulkan::new(&window).map_err(FailedToInitVulkan::new)?,
            window,
        })
    }

    pub fn render_frame(&mut self) -> VkResult<()> {
        self.vulkan.wait_for_fence(self.vulkan.in_flight_fence(), u64::MAX, true)?;

        let image_index = self.vulkan.acquire_next_image(
            u64::MAX, self.vulkan.image_available_semaphore(), vk::Fence::null(),
        )?;

        self.vulkan.reset_command_buffer()?;
        self.vulkan.record_command_buffer(image_index)?;

        self.vulkan.submit_command_buffer()?;
        self.vulkan.present_image(image_index)
    }

    pub fn handle_event(&mut self, event: WindowEvent) {
        let (_, _) = (self, event);
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
