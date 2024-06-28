use crate::vulkan::Vulkan;
use crate::const_str_to_cstr;

use ash::vk;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use std::ffi::CStr;

pub const ENGINE_NAME: &str = "HitchHiker's Engine";
pub const ENGINE_NAME_CSTR: &CStr = const_str_to_cstr!(ENGINE_NAME);

pub const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

#[derive(Default)]
pub struct Engine {
    window: Option<Window>,
    vulkan: Option<Vulkan>,
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() || event_loop.exiting() {
            return;
        }

        let window_attributes = Window::default_attributes()
            .with_title(ENGINE_NAME);
        let window = event_loop.create_window(window_attributes)
            .expect("Failed to create window");

        let vulkan = Vulkan::new(&window)
            .expect("Failed to init vulkan");

        self.vulkan = Some(vulkan);
        self.window = Some(window);
    }

    fn window_event(&mut self,
                    event_loop: &ActiveEventLoop,
                    _id: WindowId,
                    event: WindowEvent) {
        let Some(window) = &self.window else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                self.window = None;
            }
            WindowEvent::RedrawRequested => {
                window.request_redraw();
            }
            _ => ()
        }
    }
}
