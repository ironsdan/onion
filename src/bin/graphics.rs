use std::error::Error;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use onion::graphics::{
    context::GraphicsContext,
    pipelines::basic::{BasicPSO, Vert},
    render_pass::msaa::Pass,
};

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();

    let mut gfx = GraphicsContext::new(&event_loop);

    let triangle_verts = [
        Vert {
            position: [-0.5, -0.25],
            color: [0.5, 0.2, 0.2],
        },
        Vert {
            position: [0.0, 0.5],
            color: [0.5, 0.2, 0.0],
        },
        Vert {
            position: [0.25, -0.1],
            color: [0.5, 0.2, 0.4],
        },
    ];

    let triangle_vert_buf = Buffer::from_iter(
        gfx.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        triangle_verts,
    )
    .unwrap();

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let basic_pso = BasicPSO::new(
        gfx.gfx_queue.clone(),
        gfx.msaa_render_pass.draw_pass(),
        gfx.cb_allocator.clone(),
    );

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
                let image_extent: [u32; 2] = gfx.window.inner_size().into();

                if image_extent.contains(&0) {
                    return;
                }

                if gfx.recreate_swapchain {
                    gfx.recreate_swapchain();
                    viewport.extent = [image_extent[0] as f32, image_extent[1] as f32];
                }

                let future = gfx.start_frame().unwrap();

                let mut frame = gfx
                    .msaa_render_pass
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
                        Pass::Basic(mut draw_pass) => {
                            let cb = basic_pso
                                .draw(draw_pass.viewport_dimensions(), triangle_vert_buf.clone());
                            draw_pass.execute(cb).unwrap();
                        }
                        Pass::Finished(af) => {
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
