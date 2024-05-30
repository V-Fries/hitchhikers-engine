use crate::vulkan::Vulkan;
use crate::const_str_to_cstr;

use ash::vk;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasDisplayHandle};
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
        self.window = Some(event_loop.create_window(window_attributes)
            .expect("Failed to create window"));
        let display_handle = unsafe { self.window.as_ref().unwrap_unchecked() }
            .display_handle().expect("Failed to get display handle from window");
        self.vulkan = Some(Vulkan::new(display_handle.into())
            .expect("Failed to init vulkan"));
    }

    fn window_event(&mut self,
                    event_loop: &ActiveEventLoop,
                    _id: WindowId,
                    event: WindowEvent) {
        if self.window.is_none() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                self.window = None;
            }
            WindowEvent::RedrawRequested => {
                let window = unsafe { self.window.as_ref().unwrap_unchecked() };
                window.request_redraw();
            }
            _ => ()
        }
    }
}
