use camera::{Camera, PerspectiveCamera};
use glam::Vec3;
use std::{error::Error, sync::Arc};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferBeginInfo, CommandBufferLevel,
        CommandBufferUsage, RecordingCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
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
    swapchain::{
        acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{self, GpuFuture},
    Validated, VulkanError, VulkanLibrary,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use onion::graphics::camera;
use onion::graphics::cube;
use onion::graphics::fs;
use onion::graphics::vertex;
use onion::graphics::vs;

fn main() -> Result<(), impl Error> {
    let event_loop = EventLoop::new().unwrap();
    let required_extensions = Surface::required_extensions(&event_loop).unwrap();
    let library = VulkanLibrary::new().unwrap();

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            // enabled_layers: layers,
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create Vulkan instance");

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("triangle test")
            .with_inner_size(PhysicalSize::new(512.0, 512.0))
            .build(&event_loop)
            .unwrap(),
    );

    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    };

    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.intersects(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .expect("no suitable physical device found");

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index: queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();

        let image_format = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;

        Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count.max(2),
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_DST,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .unwrap()
    };

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let mut cube0 = cube::Cube::new();
    cube0.translate_y(10.0);
    cube0.scale(10.0);
    let mut cube1 = cube::Cube::new();
    cube1.translate_z(25.0);
    cube1.translate_x(25.0);
    cube1.translate_y(5.0);
    cube1.scale(5.0);
    let mut cube2 = cube::Cube::new();
    cube2.translate_z(-30.0);
    cube2.translate_x(5.0);
    cube2.translate_y(5.0);
    cube2.scale(2.0);
    let mut cube3 = cube::Cube::new();
    cube3.translate_y(6.0);
    cube3.translate_x(-50.0);
    cube3.scale(6.0);
    let mut cube4 = cube::Cube::new();
    cube4.translate_y(3.0);
    cube4.translate_z(50.0);
    cube4.scale(3.0);
    let mut vertices = cube0.to_vec();
    vertices.append(&mut cube1.to_vec());
    vertices.append(&mut cube2.to_vec());
    vertices.append(&mut cube3.to_vec());
    vertices.append(&mut cube4.to_vec());

    let mut floor = vec![
        vertex::Vertex {
            position: Vec3::new(1000.0, 0.0, 1000.0).to_array(),
            color: Vec3::new(0.7, 0.7, 0.7).to_array(),
            ..Default::default()
        },
        vertex::Vertex {
            position: Vec3::new(1000.0, 0.0, -1000.0).to_array(),
            color: Vec3::new(0.7, 0.7, 0.7).to_array(),
            ..Default::default()
        },
        vertex::Vertex {
            position: Vec3::new(-1000.0, 0.0, -1000.0).to_array(),
            color: Vec3::new(0.7, 0.7, 0.7).to_array(),
            ..Default::default()
        },
        vertex::Vertex {
            position: Vec3::new(1000.0, 0.0, 1000.0).to_array(),
            color: Vec3::new(0.7, 0.7, 0.7).to_array(),
            ..Default::default()
        },
        vertex::Vertex {
            position: Vec3::new(-1000.0, 0.0, -1000.0).to_array(),
            color: Vec3::new(0.7, 0.7, 0.7).to_array(),
            ..Default::default()
        },
        vertex::Vertex {
            position: Vec3::new(-1000.0, 0.0, 1000.0).to_array(),
            color: Vec3::new(0.7, 0.7, 0.7).to_array(),
            ..Default::default()
        },
    ];

    vertices.append(&mut floor);

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

    // mod vs {
    //     vulkano_shaders::shader! {
    //         ty: "vertex",
    //         path: "src/shader.vert.glsl",
    //         dump: true,
    //     }
    // }

    // mod fs {
    //     vulkano_shaders::shader! {
    //         ty: "fragment",
    //         path: "src/shader.frag.glsl",
    //         dump: true,
    //     }
    // }

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            intermediary: {
                format: swapchain.image_format(),
                // This has to match the image definition.
                samples: 8,
                load_op: Clear,
                store_op: DontCare,
            },
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth_stencil: {
                format: Format::D16_UNORM,
                samples: 8,
                load_op: Clear,
                store_op: DontCare,
            },
        },
        pass: {
            color: [intermediary],
            color_resolve: [color],
            depth_stencil: {depth_stencil},
        },
    )
    .unwrap();

    let pipeline = {
        let vs = vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = vertex::Vertex::per_vertex().definition(&vs).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            // Since we only have one pipeline in this example, and thus one pipeline layout,
            // we automatically generate the creation info for it from the resources used in the
            // shaders. In a real application, you would specify this information manually so that
            // you can re-use one layout in multiple pipelines.
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                // How vertex data is read from the vertex buffers into the vertex shader.
                vertex_input_state: Some(vertex_input_state),
                // How vertices are arranged into primitive shapes.
                // The default primitive shape is a triangle.
                input_assembly_state: Some(InputAssemblyState {
                    topology: PrimitiveTopology::TriangleList,
                    ..Default::default()
                }),
                // How primitives are transformed and clipped to fit the framebuffer.
                // We use a resizable viewport, set to draw over the entire window.
                viewport_state: Some(ViewportState::default()),
                // How polygons are culled and converted into a raster of pixels.
                // The default value does not perform any culling.
                rasterization_state: Some(RasterizationState::default()),
                // How multiple fragment shader samples are converted to a single pixel value.
                // The default value does not perform any multisampling.
                multisample_state: Some(MultisampleState {
                    rasterization_samples: subpass.num_samples().unwrap(),
                    ..Default::default()
                }),
                // How pixel values are combined with the values already present in the framebuffer.
                // The default value overwrites the old value with the new one, without any
                // blending.
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                // Dynamic states allows us to specify parts of the pipeline settings when
                // recording the command buffer, before we perform drawing.
                // Here, we specify that the viewport should be dynamic.
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };

    // Dynamic viewports allow us to recreate just the viewport when the window is resized.
    // Otherwise we would have to recreate the whole pipeline.
    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let mut framebuffers = window_size_dependent_setup(
        &images,
        render_pass.clone(),
        &mut viewport,
        memory_allocator.clone(),
        swapchain.image_format(),
    );

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    let image_extent: [u32; 2] = window.inner_size().into();
    println!(
        "aspect ratio: {}",
        image_extent[0] as f32 / image_extent[1] as f32
    );
    let mut camera = PerspectiveCamera::new(
        80.0,
        image_extent[0] as f32 / image_extent[1] as f32,
        1.5,
        500.0,
    );

    // camera.translate_y(-2.0);
    // camera.rotate_z(cgmath::Deg(-35.0));
    // camera.rotate_y(cgmath::Deg(-25.0));
    camera.translate_z(-20.0);

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
                let image_extent: [u32; 2] = window.inner_size().into();
                recreate_swapchain = true;
                println!(
                    "aspect ratio: {}",
                    image_extent[0] as f32 / image_extent[1] as f32
                );
                camera.set_aspect_ratio(image_extent[0] as f32 / image_extent[1] as f32);
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                println!("{:?}", event);
                if event.physical_key == PhysicalKey::Code(KeyCode::Minus) {
                    camera.translate_z(-0.5);
                } else if event.physical_key == PhysicalKey::Code(KeyCode::Equal) {
                    camera.translate_z(0.5);
                } else if event.physical_key == PhysicalKey::Code(KeyCode::ArrowUp) {
                    camera.translate_y(0.5);
                } else if event.physical_key == PhysicalKey::Code(KeyCode::ArrowDown) {
                    camera.translate_y(-0.5);
                } else if event.physical_key == PhysicalKey::Code(KeyCode::ArrowLeft) {
                    camera.translate_x(-0.5);
                } else if event.physical_key == PhysicalKey::Code(KeyCode::ArrowRight) {
                    camera.translate_x(0.5);
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyA) {
                    camera.rotate_y(cgmath::Deg(-5.0));
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyD) {
                    camera.rotate_y(cgmath::Deg(5.0));
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyW) {
                    camera.rotate_x(cgmath::Deg(-5.0));
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyS) {
                    camera.rotate_x(cgmath::Deg(5.0));
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyQ) {
                    camera.rotate_z(cgmath::Deg(-5.0));
                } else if event.physical_key == PhysicalKey::Code(KeyCode::KeyE) {
                    camera.rotate_z(cgmath::Deg(5.0));
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let image_extent: [u32; 2] = window.inner_size().into();

                if image_extent.contains(&0) {
                    return;
                }

                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if recreate_swapchain {
                    let (new_swapchain, new_images) = swapchain
                        .recreate(SwapchainCreateInfo {
                            image_extent,
                            ..swapchain.create_info()
                        })
                        .expect("failed to recreate swapchain");

                    swapchain = new_swapchain;

                    framebuffers = window_size_dependent_setup(
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                        memory_allocator.clone(),
                        swapchain.image_format(),
                    );

                    recreate_swapchain = false;
                }

                let (image_index, suboptimal, acquire_future) =
                    match acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap) {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("failed to acquire next image: {e}"),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                let mut builder = RecordingCommandBuffer::new(
                    command_buffer_allocator.clone(),
                    queue.queue_family_index(),
                    CommandBufferLevel::Primary,
                    CommandBufferBeginInfo {
                        usage: CommandBufferUsage::OneTimeSubmit,
                        ..Default::default()
                    },
                )
                .unwrap();

                // let angle = 0.0;
                // let radians = angle * (std::f32::consts::PI / 180.0);

                // println!("deg: {}, rad: {}", angle, radians);

                // let rotate_x = Mat4::from_rotation_x(radians);
                // let rotate_y = Mat4::from_rotation_y(radians);
                // // let scale = Mat4::from_scale(Vec3::new(0.25, 0.25, 0.25));\
                // let translation = Mat4::from_translation(Vec3::new(0.1, 0.1, -0.1));
                // let f = (rotate_x * rotate_y) + translation;
                // let f = translation;
                // let mut camera = PerspectiveCamera::default();

                // camera.translate_z(10.0);

                // let f = camera.mvp_mat();

                // println!("{}", f);

                let push_constants = vs::constants {
                    data: [0.5, 0.5, image_extent[0] as f32, image_extent[1] as f32],
                    render_matrix: camera.mvp_mat().to_cols_array_2d(),
                };

                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![
                                Some([0.0, 0.0, 0.0, 1.0].into()),
                                Some([0.0, 0.0, 0.0, 1.0].into()),
                                Some(1f32.into()),
                            ],

                            ..RenderPassBeginInfo::framebuffer(
                                framebuffers[image_index as usize].clone(),
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

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    // The color output is now expected to contain our triangle. But in order to
                    // show it on the screen, we have to *present* the image by calling
                    // `then_swapchain_present`.
                    //
                    // This function does not actually present the image immediately. Instead it
                    // submits a present command at the end of the queue. This means that it will
                    // only be presented once the GPU has finished executing the command buffer
                    // that draws the triangle.
                    .then_swapchain_present(
                        queue.clone(),
                        SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
                    )
                    .then_signal_fence_and_flush();

                match future.map_err(Validated::unwrap) {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        panic!("failed to flush future: {e}");
                    }
                }
            }
            Event::AboutToWait => window.request_redraw(),
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
                samples: SampleCount::Sample8,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    let depth_buffer = ImageView::new_default(
        Image::new(
            memory_allocator,
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: images[0].extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                samples: SampleCount::Sample8,
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
                    attachments: vec![intermediary.clone(), view, depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
