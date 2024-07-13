use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::engine::Engine;

#[derive(Default)]
pub struct App {
    engine: Option<Engine>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_some() || event_loop.exiting() {
            return;
        }

        let engine = Engine::new(event_loop)
            .expect("Failed to init Engine");
        engine.window().request_redraw();
        self.engine = Some(engine);
    }

    fn window_event(&mut self,
                    event_loop: &ActiveEventLoop,
                    _id: WindowId,
                    event: WindowEvent) {
        let Some(engine) = &mut self.engine else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                self.engine = None;
            }
            WindowEvent::RedrawRequested => {
                engine.render_frame()
                    .expect("Failed to render a frame");
                engine.window().request_redraw();
            }
            _ => engine.handle_event(event)
        }
    }
}
