use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferBeginInfo, CommandBufferLevel,
        CommandBufferUsage, RecordingCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassContents,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    pipeline::GraphicsPipeline,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
    Validated, VulkanError,
};

use crate::graphics::pipelines;

pub struct Pass {
    pub render_pass: Arc<RenderPass>,
    pub line_pso: Arc<GraphicsPipeline>,
    pub texture_pso: Arc<GraphicsPipeline>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
}

impl Pass {
    pub fn new_msaa_render_pass(
        device: Arc<Device>,
        images: &[Arc<Image>],
        format: Format,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Pass, Validated<VulkanError>> {
        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                intermediary: {
                    format: format,
                    // This has to match the image definition.
                    samples: 4,
                    load_op: Clear,
                    store_op: DontCare,
                },
                color: {
                    format: format,
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
        )?;

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let line_pso = pipelines::line::line_pso(device.clone(), subpass.clone())?;
        let texture_pso = pipelines::texture::texture_pso(device.clone(), subpass.clone())?;

        let framebuffers = window_size_dependent_setup(
            images,
            render_pass.clone(),
            memory_allocator.clone(),
            format,
        );

        Ok(Self {
            render_pass,
            line_pso,
            texture_pso,
            framebuffers,
        })
    }

    pub fn window_size_update(
        &mut self,
        images: &[Arc<Image>],
        format: Format,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) {
        self.framebuffers = window_size_dependent_setup(
            images,
            self.render_pass.clone(),
            memory_allocator.clone(),
            format,
        );
    }

    pub fn start(
        &mut self,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        queue: Arc<Queue>,
        image_index: u32,
    ) -> RecordingCommandBuffer {
        let mut command_buffer = RecordingCommandBuffer::new(
            command_buffer_allocator.clone(),
            queue.queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
        .unwrap();
        command_buffer
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.7, 0.7, 0.7, 1.0].into()),
                        Some([0.7, 0.7, 0.7, 1.0].into()),
                    ],

                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap();

        return command_buffer;
    }

    pub fn finish(
        &mut self,
        future: Box<dyn GpuFuture>,
        queue: Arc<Queue>,
        mut cb: RecordingCommandBuffer,
    ) -> Box<dyn GpuFuture> {
        cb.end_render_pass(Default::default()).unwrap();

        let command_buffer = cb.end().unwrap();
        future
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .boxed()
    }
}

/// This function is called once during initialization, then again whenever the window is resized.
fn window_size_dependent_setup(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    format: Format,
) -> Vec<Arc<Framebuffer>> {
    let extent = images[0].extent();
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
