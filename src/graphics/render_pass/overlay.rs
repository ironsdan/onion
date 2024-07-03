use std::sync::Arc;

use vulkano::{
    command_buffer::{
        self, allocator::StandardCommandBufferAllocator, CommandBufferBeginInfo,
        CommandBufferLevel, CommandBufferUsage, RecordingCommandBuffer, RenderPassBeginInfo,
        SubpassBeginInfo, SubpassContents,
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
    pub framebuffers: Vec<Arc<Framebuffer>>,
}

impl Pass {
    pub fn new_overlay_render_pass(
        device: Arc<Device>,
        images: &[Arc<Image>],
        format: Format,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Pass, Validated<VulkanError>> {
        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: format,
                    samples: 1,
                    load_op: Load,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )?;

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let line_pso = pipelines::line::line_pso(device.clone(), subpass.clone())?;
        let framebuffers = window_size_dependent_setup(
            images,
            render_pass.clone(),
            memory_allocator.clone(),
            format,
        );

        Ok(Self {
            render_pass,
            line_pso,
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
                    clear_values: vec![None],

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
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
