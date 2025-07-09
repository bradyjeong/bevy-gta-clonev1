//! Minimal example demonstrating the Amp game engine
//!
//! This example opens a window and renders a purple screen using the amp_gpu crate.

use amp_gpu::{GpuContext, SurfaceManager};
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Amp Game Engine - Minimal Example")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .build(&event_loop)?,
    );

    // Create GPU context
    let gpu_context = pollster::block_on(GpuContext::new(&window))?;

    // Create surface manager
    let mut surface_manager = SurfaceManager::new(&gpu_context, &window, window.inner_size())?;

    println!("Adapter: {:?}", gpu_context.adapter_info());
    println!("Features: {:?}", gpu_context.device_features());
    println!("Surface format: {:?}", surface_manager.format());

    let window_id = window.id();
    let window_for_loop = Arc::clone(&window);

    // Main event loop
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id: event_window_id,
            } if event_window_id == window_id => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        surface_manager.resize(&gpu_context, *physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        // Render purple frame
                        match surface_manager.render_purple_frame(&gpu_context) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Render error: {e}"),
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // Request a redraw
                window_for_loop.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
