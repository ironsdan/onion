use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::CommandBuffer,
    image::Image,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
};

use super::pipelines::texture::PSOTexture;
use super::pipelines::texture::Vert;

pub struct Texture {
    size: f32,
}

impl Texture {
    pub fn new(size: f32) -> Self {
        Texture { size }
    }

    pub fn draw(
        &self,
        memory_allocator: Arc<dyn MemoryAllocator>,
        pipeline: &mut PSOTexture,
        image: Arc<Image>,
        viewport: [u32; 2],
    ) -> Arc<CommandBuffer> {
        let vertices = [
            Vert {
                position: [-self.size, -self.size],
            },
            Vert {
                position: [self.size, self.size],
            },
            Vert {
                position: [-self.size, self.size],
            },
            Vert {
                position: [-self.size, -self.size],
            },
            Vert {
                position: [self.size, -self.size],
            },
            Vert {
                position: [self.size, self.size],
            },
        ];

        let vb = Buffer::from_iter(
            memory_allocator,
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

        pipeline.draw(viewport, image, vb)
    }
}
