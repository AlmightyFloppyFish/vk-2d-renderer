use std::sync::Arc;

use crate::renderer::main::draw;
use crate::renderer::vertex::Vertex;
use crate::renderer::DrawGraphicsPipeline;
use image::GenericImageView;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::DescriptorSet;
use vulkano::device;
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImmutableImage};

pub trait Entity {
    fn init(&mut self); // TODO: These should also have a matrix as param
    fn update(&mut self); // TODO: I can also give them `game` to fork
}

pub struct Texture {
    pub unloaded: Vec<u8>,
    pub entity: Arc<Entity>,
    pub matrix: Matrix,
    pub loaded: Option<Arc<DescriptorSet + Send + Sync>>,
    pub waiter: Option<TextureLoadAwait>,
    pub dimensions: (u32, u32),
}

pub struct Matrix {
    pub pos: (f32, f32),
    pub size: (f32, f32),
}

impl Matrix {
    pub fn new(pos: (f32, f32), size: (f32, f32)) -> Self {
        Matrix {
            pos: pos,
            size: size,
        }
    }
}

impl Texture {
    // Impl this for Texture instead so i can use the dimensions field
    pub fn to_vert(self: &Self, screen: (u32, u32)) -> [Vertex; 4] {
        let scale_x = screen.0 as f32 / self.dimensions.0 as f32;
        let scale_y = screen.1 as f32 / self.dimensions.1 as f32;
        let scale = (scale_x, scale_y);
        [
            Vertex {
                // Top-Left
                position: [self.matrix.pos.0, self.matrix.pos.1],
                scale: scale,
            },
            Vertex {
                // Bottom-Left
                position: [self.matrix.pos.0, self.matrix.pos.1 + self.matrix.size.1],
                scale: scale,
            },
            Vertex {
                // Top-Right
                position: [self.matrix.pos.0 + self.matrix.size.0, self.matrix.pos.1],
                scale: scale,
            },
            Vertex {
                // Bottom-Right
                position: [
                    self.matrix.pos.0 + self.matrix.size.0,
                    self.matrix.pos.1 + self.matrix.size.1,
                ],
                scale: scale,
            },
        ]
    }
}

type TextureLoadAwait = vulkano::command_buffer::CommandBufferExecFuture<
    vulkano::sync::NowFuture,
    vulkano::command_buffer::AutoCommandBuffer,
>;

impl Texture {
    pub fn load_gpu(
        &mut self,
        queue: Arc<device::Queue>,
        device: Arc<device::Device>,
        pipeline: Arc<DrawGraphicsPipeline>,
    ) {
        let img = image::load_from_memory(&self.unloaded)
            .unwrap_or_else(|e| panic!("Unable to load image {}", e));
        let dims = img.dimensions();
        self.dimensions = dims;

        let (tex, fut) = ImmutableImage::from_iter(
            img.to_rgba().into_raw().iter().cloned(),
            Dimensions::Dim2d {
                width: dims.0,
                height: dims.1,
            },
            Format::R8G8B8A8Srgb,
            queue,
        )
        .unwrap();

        let sampler = draw::default_sampler(device.clone());

        let set = Arc::new(
            PersistentDescriptorSet::start(pipeline, 0)
                .add_sampled_image(tex, sampler)
                .unwrap()
                .build()
                .unwrap(),
        );
        self.loaded = Some(set);
        self.waiter = Some(fut)
    }
}
