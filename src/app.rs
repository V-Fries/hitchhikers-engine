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

        // TODO try to self.exit(event_loop) instead of expect
        let engine = Engine::new(event_loop).expect("Failed to init Engine");
        engine.window().request_redraw();
        self.engine = Some(engine);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(engine) = &mut self.engine else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                self.exit(event_loop);
            }
            WindowEvent::RedrawRequested => {
                if let Err(err) = engine.render_frame() {
                    eprintln!("Failed to render frame: {err}");
                    self.exit(event_loop);
                    return;
                }
                engine.window().request_redraw();
            }
            _ => {
                if let Err(err) = engine.handle_event(&event) {
                    eprintln!("Failed to handle event ({event:?}): {err}");
                    self.exit(event_loop);

                    #[allow(dead_code)]
                    // This is here to remove the warning that says the return is useless
                    // I want to keep the return because I might add more code after the match
                    // later
                    return;
                }
            }
        }
    }
}

impl App {
    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.exit();
        if let Some(mut engine) = self.engine.take() {
            unsafe { engine.destroy() };
        }
    }
}
