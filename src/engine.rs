mod errors;

use crate::engine::errors::{FailedToCreateWindow, FailedToInitVulkan};
use crate::vulkan_renderer::VulkanRenderer;
use ash::vk;
use rs42::const_str_to_cstr;
use rs42::Result;
use std::ffi::CStr;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub const ENGINE_NAME: &str = "Hitchhiker's Engine";
pub const ENGINE_NAME_CSTR: &CStr = const_str_to_cstr!(ENGINE_NAME);

pub const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

pub struct Engine {
    vulkan_renderer: VulkanRenderer,
    window: Window,
}

impl Engine {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let window_attributes = Window::default_attributes().with_title(ENGINE_NAME);

        let window = event_loop
            .create_window(window_attributes)
            .map_err(FailedToCreateWindow::new)?;

        Ok(Self {
            vulkan_renderer: VulkanRenderer::new(&window).map_err(FailedToInitVulkan::new)?,
            window,
        })
    }

    pub fn render_frame(&mut self) -> Result<()> {
        self.vulkan_renderer.render_frame(&self.window)
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> Result<()> {
        match event {
            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                inner_size_writer: _,
            }
            | WindowEvent::Resized(_) => {
                // TODO maybe handle minimization differently?
                unsafe { self.vulkan_renderer.recreate_swapchain(&self.window) }
            }
            _ => Ok(()),
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub unsafe fn destroy(&mut self) {
        self.vulkan_renderer.destroy();
    }
}
