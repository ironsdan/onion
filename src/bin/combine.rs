use onion::graphics::context::GraphicsContext;
use std::error::Error;
use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsage, CopyBufferToImageInfo,
    RecordingCommandBuffer,
};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::format::Format;
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{Pipeline, PipelineBindPoint};
use vulkano::sync::GpuFuture;
use vulkano::DeviceSize;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use onion::graphics::pipelines::line::{vs, Vert};

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();

    let mut gfx = GraphicsContext::new(&event_loop);

    let box_verts = [
        Vert {
            position: [-0.1, -0.1],
        },
        Vert {
            position: [-0.1, 0.1],
        },
        Vert {
            position: [-0.1, 0.1],
        },
        Vert {
            position: [0.1, 0.1],
        },
        Vert {
            position: [0.1, -0.1],
        },
        Vert {
            position: [0.1, 0.1],
        },
        Vert {
            position: [-0.1, -0.1],
        },
        Vert {
            position: [0.1, -0.1],
        },
    ];

    let line_vert_buf = Buffer::from_iter(
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
        box_verts,
    )
    .unwrap();

    let tex_verts = [
        Vert {
            position: [-0.5, -0.5],
        },
        Vert {
            position: [-0.5, 0.5],
        },
        Vert {
            position: [0.5, -0.5],
        },
        Vert {
            position: [0.5, 0.5],
        },
    ];

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(gfx.device.clone()));

    let tex_vert_buf = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        tex_verts,
    )
    .unwrap();

    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        gfx.device.clone(),
        Default::default(),
    ));
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        gfx.device.clone(),
        Default::default(),
    ));

    let mut uploads = RecordingCommandBuffer::new(
        command_buffer_allocator.clone(),
        gfx.graphics_queue.queue_family_index(),
        CommandBufferLevel::Primary,
        CommandBufferBeginInfo {
            usage: CommandBufferUsage::OneTimeSubmit,
            ..Default::default()
        },
    )
    .unwrap();

    let texture = {
        let png_bytes = include_bytes!("img.png").as_slice();
        let decoder = png::Decoder::new(png_bytes);
        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();
        let extent = [info.width, info.height, 1];

        let upload_buffer = Buffer::new_slice(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            (info.width * info.height * 4) as DeviceSize,
        )
        .unwrap();

        reader
            .next_frame(&mut upload_buffer.write().unwrap())
            .unwrap();

        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_SRGB,
                extent,
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap();

        uploads
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                upload_buffer,
                image.clone(),
            ))
            .unwrap();

        ImageView::new_default(image).unwrap()
    };

    let sampler = Sampler::new(
        gfx.device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            address_mode: [SamplerAddressMode::Repeat; 3],
            ..Default::default()
        },
    )
    .unwrap();

    let layout = &gfx.msaa_render_pass.texture_pso.layout().set_layouts()[0];
    let set = DescriptorSet::new(
        descriptor_set_allocator,
        layout.clone(),
        [
            WriteDescriptorSet::sampler(0, sampler),
            WriteDescriptorSet::image_view(1, texture),
        ],
        [],
    )
    .unwrap();

    gfx.previous_frame_end = Some(
        uploads
            .end()
            .unwrap()
            .execute(gfx.graphics_queue.clone())
            .unwrap()
            .boxed(),
    );

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let mut mouse_pos: [f32; 2] = [0.0, 0.0];

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
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let extent = gfx.window.inner_size();
                mouse_pos = [
                    (2.0 * position.x as f32 - (extent.width as f32)) / (extent.width as f32),
                    (2.0 * position.y as f32 - (extent.height as f32)) / (extent.height as f32),
                ];
                println!("{:?}", mouse_pos);
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

                let mut cb = gfx.msaa_render_pass.start(
                    gfx.command_buffer_allocator.clone(),
                    gfx.graphics_queue.clone(),
                    gfx.image_index,
                );

                cb.set_viewport(0, [viewport.clone()].into_iter().collect())
                    .unwrap()
                    .bind_pipeline_graphics(gfx.msaa_render_pass.texture_pso.clone())
                    .unwrap()
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        gfx.msaa_render_pass.texture_pso.layout().clone(),
                        0,
                        set.clone(),
                    )
                    .unwrap()
                    .bind_vertex_buffers(0, tex_vert_buf.clone())
                    .unwrap();

                unsafe {
                    cb
                        // We add a draw command.
                        .draw(tex_vert_buf.len() as u32, 1, 0, 0)
                        .unwrap();
                }

                let after1 = gfx
                    .msaa_render_pass
                    .finish(future, gfx.graphics_queue.clone(), cb);

                let mut cb = gfx.overlay_render_pass.start(
                    gfx.command_buffer_allocator.clone(),
                    gfx.graphics_queue.clone(),
                    gfx.image_index,
                );

                println!("{:?}", mouse_pos);
                let push_constants = vs::constants { mouse_pos };

                cb.set_viewport(0, [viewport.clone()].into_iter().collect())
                    .unwrap()
                    .push_constants(
                        gfx.overlay_render_pass.line_pso.layout().clone(),
                        0,
                        push_constants,
                    )
                    .unwrap()
                    .bind_pipeline_graphics(gfx.overlay_render_pass.line_pso.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, line_vert_buf.clone())
                    .unwrap();

                unsafe {
                    cb.draw(line_vert_buf.len() as u32, 1, 0, 0).unwrap();
                }
                cb.end_render_pass(Default::default()).unwrap();
                let command_buffer = cb.end().unwrap();
                let after2 = after1
                    .then_execute(gfx.graphics_queue.clone(), command_buffer)
                    .unwrap()
                    .then_signal_fence_and_flush()
                    .unwrap()
                    .boxed();

                gfx.finish_frame(Box::new(after2));
            }
            Event::AboutToWait => gfx.window.request_redraw(),
            _ => (),
        }
    })
}
