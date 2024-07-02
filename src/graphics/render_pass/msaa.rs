use std::sync::Arc;

use vulkano::{
    device::Device,
    format::Format,
    pipeline::GraphicsPipeline,
    render_pass::{RenderPass, Subpass},
    Validated, VulkanError,
};

use crate::graphics::pipelines;

pub struct Pass {
    pub render_pass: Arc<RenderPass>,
    pub line_pso: Arc<GraphicsPipeline>,
    pub texture_pso: Arc<GraphicsPipeline>,
}

impl Pass {
    pub fn new_msaa_render_pass(
        device: Arc<Device>,
        format: Format,
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

        Ok(Self {
            render_pass,
            line_pso,
            texture_pso,
        })
    }
}
