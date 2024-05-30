mod vulkan;
mod engine;
mod utils;

use engine::Engine;

use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut engine = Engine::default();

    event_loop.run_app(&mut engine)
}
