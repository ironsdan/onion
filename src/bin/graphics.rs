use onion::graphics::{
    context::GraphicsContext,
    render_pass::basic::{BasicMSAAPass, BasicPass},
    shape, Color,
};
use std::error::Error;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use onion::graphics::texture::Texture;

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();

    let mut gfx = GraphicsContext::new(&event_loop);

    let png_bytes = include_bytes!("img.png").as_slice();
    let buf_info = gfx.upload_png(png_bytes);
    let image = gfx.upload_image(buf_info.0, buf_info.1);

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

                let render_pass = &mut gfx.render_passes.basic_msaa;
                let basic_pipeline = &mut gfx.pipelines.basic;
                let texture_pipeline = &mut gfx.pipelines.texture;

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
                        BasicMSAAPass::Draw(mut draw_pass) => {
                            let img = Texture::new(0.5);
                            let cb = img.draw(
                                gfx.memory_allocator.clone(),
                                texture_pipeline,
                                image.clone(),
                                draw_pass.viewport_dimensions(),
                            );
                            draw_pass.execute(cb).unwrap();
                            let square = shape::Square::new(0.1, Color::red());
                            let cb = square.draw(
                                gfx.memory_allocator.clone(),
                                basic_pipeline,
                                draw_pass.viewport_dimensions(),
                            );
                            draw_pass.execute(cb).unwrap();
                        }
                        BasicMSAAPass::Finished(af) => {
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
