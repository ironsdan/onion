use onion::graphics::{
    context::GraphicsContext,
    render_pass::{basic::BasicMSAAPass, overlay::OverlayPass},
    shape, Color,
};
use std::error::Error;
use vulkano::sync::future::GpuFuture;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use onion::graphics::texture::Texture;

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();

    let mut gfx = GraphicsContext::new(&event_loop);

    // Read the font data.
    let font = include_bytes!("Roboto-Regular.ttf") as &[u8];
    // Parse it into the font type.
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    // Rasterize and get the layout metrics for the letter 'g' at 17px.
    let (metrics, bitmap) = font.rasterize('D', 17.0);
    println!("{:?}\n{:?}", metrics, bitmap);

    let mut buf: Vec<u8> = Vec::new();
    for val in bitmap {
        buf.push(0);
        buf.push(0);
        buf.push(0);
        buf.push(val);
    }

    println!("{:?}", buf);

    let image = gfx.upload_rgba(buf, [metrics.width as u32, metrics.height as u32, 1]);

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
                let overlay_pipeline = &mut gfx.pipelines.overlay;

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

                let after1 = after_future.unwrap().then_signal_fence_and_flush().unwrap();

                let render_pass = &mut gfx.render_passes.overlay;
                let mut frame = render_pass
                    .frame(
                        after1,
                        gfx.final_images[gfx.image_index as usize].clone(),
                        gfx.memory_allocator.clone(),
                    )
                    .unwrap();

                let mut after_future2 = None;
                while let Some(pass) = frame.next_pass().unwrap() {
                    match pass {
                        OverlayPass::Draw(mut draw_pass) => {
                            let square = shape::Square::new(0.1, Color::red());
                            let cb = square.draw(
                                gfx.memory_allocator.clone(),
                                overlay_pipeline,
                                draw_pass.viewport_dimensions(),
                            );
                            draw_pass.execute(cb).unwrap();
                        }
                        OverlayPass::Finished(af) => {
                            after_future2 = Some(af);
                        }
                    }
                }

                gfx.finish_frame(after_future2.unwrap());
            }
            Event::AboutToWait => gfx.window.request_redraw(),
            _ => (),
        }
    })
}
