use onion::graphics::{context::GraphicsContext, render_pass::basic::BasicPass, shape, Color};
use std::error::Error;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();

    let mut gfx = GraphicsContext::new(&event_loop);

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                gfx.recreate_swapchain = true;
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let future = gfx.start_frame().unwrap();

                let render_pass = &mut gfx.render_passes.basic;
                let pipeline = &mut gfx.pipelines.basic;

                let mut frame = render_pass
                    .frame(
                        [0.7, 0.7, 0.7, 1.0],
                        future,
                        gfx.final_images[gfx.image_index as usize].clone(),
                        gfx.memory_allocator.clone(),
                    )
                    .unwrap();

                let mut after_future = None;
                while let Some(pass) = frame.next_pass().unwrap() {
                    match pass {
                        BasicPass::Draw(mut draw_pass) => {
                            let square = shape::Square::new(0.1, Color::red());
                            let cb = square.draw(
                                gfx.memory_allocator.clone(),
                                pipeline,
                                draw_pass.viewport_dimensions(),
                            );
                            draw_pass.execute(cb).unwrap();
                        }
                        BasicPass::Finished(af) => {
                            after_future = Some(af);
                        }
                    }
                }

                gfx.finish_frame(after_future.unwrap());
            }
            Event::AboutToWait => gfx.window.request_redraw(),
            _ => (),
        }
    })
}
