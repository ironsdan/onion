use onion::graphics::context::GraphicsContext;
use std::{error::Error, sync::Arc};
use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferBeginInfo, CommandBufferLevel,
        CommandBufferUsage, RecordingCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();

    let mut gfx = GraphicsContext::new(&event_loop);

    #[derive(BufferContents, vertex_input::Vertex)]
    #[repr(C)]
    pub struct Vertex {
        #[format(R32G32_SFLOAT)]
        pub position: [f32; 2],
    }

    let vertices = [
        Vertex {
            position: [-0.1, -0.1],
        },
        Vertex {
            position: [-0.1, 0.1],
        },
        Vertex {
            position: [-0.1, 0.1],
        },
        Vertex {
            position: [0.1, 0.1],
        },
        Vertex {
            position: [0.1, -0.1],
        },
        Vertex {
            position: [0.1, 0.1],
        },
        Vertex {
            position: [-0.1, -0.1],
        },
        Vertex {
            position: [0.1, -0.1],
        },
    ];

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(gfx.device.clone()));

    let vertex_buffer = Buffer::from_iter(
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
        vertices,
    )
    .unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        gfx.device.clone(),
        attachments: {
            intermediary: {
                format: gfx.swapchain.image_format(),
                // This has to match the image definition.
                samples: 4,
                load_op: Clear,
                store_op: DontCare,
            },
            color: {
                format: gfx.swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [intermediary],
            color_resolve: [color],
            depth_stencil: {},
        },
    )
    .unwrap();

    let pipeline = {
        let vs = vs::load(gfx.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(gfx.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = Vertex::per_vertex().definition(&vs).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            gfx.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(gfx.device.clone())
                .unwrap(),
        )
        .unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        GraphicsPipeline::new(
            gfx.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState {
                    topology: PrimitiveTopology::LineList,
                    ..Default::default()
                }),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState {
                    rasterization_samples: subpass.num_samples().unwrap(),
                    ..Default::default()
                }),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState {
                        blend: Some(AttachmentBlend::alpha()),
                        ..Default::default()
                    },
                )),
                depth_stencil_state: None,
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let mut framebuffers = window_size_dependent_setup(
        &gfx.final_images,
        render_pass.clone(),
        &mut viewport,
        memory_allocator.clone(),
        gfx.swapchain.image_format(),
    );

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        gfx.device.clone(),
        Default::default(),
    ));

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
                    (position.x as f32 - (extent.width / 2) as f32) / ((extent.width / 2) as f32),
                    (position.y as f32 - (extent.height / 2) as f32) / ((extent.height / 2) as f32),
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
                    framebuffers = window_size_dependent_setup(
                        &gfx.final_images,
                        render_pass.clone(),
                        &mut viewport,
                        memory_allocator.clone(),
                        gfx.swapchain.image_format(),
                    );
                }

                let future = gfx.start_frame().unwrap();

                let mut builder = RecordingCommandBuffer::new(
                    command_buffer_allocator.clone(),
                    gfx.graphics_queue.queue_family_index(),
                    CommandBufferLevel::Primary,
                    CommandBufferBeginInfo {
                        usage: CommandBufferUsage::OneTimeSubmit,
                        ..Default::default()
                    },
                )
                .unwrap();

                println!("{:?}", mouse_pos);
                let push_constants = vs::constants { mouse_pos };

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0.7, 0.7, 0.7, 1.0].into()),
                                Some([0.7, 0.7, 0.7, 1.0].into()),
                            ],

                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[gfx.image_index as usize].clone(),
                            )
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .set_viewport(0, [viewport.clone()].into_iter().collect())
                    .unwrap()
                    .push_constants(pipeline.layout().clone(), 0, push_constants)
                    .unwrap()
                    .bind_pipeline_graphics(pipeline.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .unwrap();

                unsafe {
                    builder
                        // We add a draw command.
                        .draw(vertex_buffer.len() as u32, 1, 0, 0)
                        .unwrap();
                }

                builder.end_render_pass(Default::default()).unwrap();

                let command_buffer = builder.end().unwrap();
                let after = future
                    .then_execute(gfx.graphics_queue.clone(), command_buffer)
                    .unwrap();

                gfx.finish_frame(Box::new(after));
            }
            Event::AboutToWait => gfx.window.request_redraw(),
            _ => (),
        }
    })
}

/// This function is called once during initialization, then again whenever the window is resized.
fn window_size_dependent_setup(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
    memory_allocator: Arc<StandardMemoryAllocator>,
    format: Format,
) -> Vec<Arc<Framebuffer>> {
    let extent = images[0].extent();
    viewport.extent = [extent[0] as f32, extent[1] as f32];

    let intermediary = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: format,
                extent: [extent[0], extent[1], 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                samples: SampleCount::Sample4,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![intermediary.clone(), view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(push_constant) uniform constants {
                vec2 mouse_pos;
            } pc;

            layout(location = 0) in vec2 position;
            layout(location = 0) out vec3 v_color;

            void main() {
                vec2 p = position + pc.mouse_pos;
                gl_Position = vec4(p, 0.0, 1.0);
                v_color = vec3(0.0, 0.0, 0.0);
            }
        ",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) in vec3 v_color;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(v_color, 1.0);
            }
        ",
    }
}
