pub(crate) mod entity;
mod init;
mod main;
pub mod shader;
pub mod vertex;

use entity::{Entity, Matrix, Texture};
use hashbrown::HashMap;
use std::sync::{Arc, Mutex};
use vulkano::command_buffer;
use vulkano::device;
use vulkano::framebuffer;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image;
use vulkano::instance;
use vulkano::pipeline;
use vulkano::swapchain;

const VSYNC: bool = true;

pub struct Game<S> {
    user_global_state: S, // RwLock?
    enabled_textures: HashMap<String, Arc<Mutex<Texture>>>,
    disabled_textures: HashMap<String, Arc<Mutex<Texture>>>,
}

impl<S> Game<S> {
    pub fn new(state: S) -> Self {
        Game {
            enabled_textures: HashMap::new(),
            disabled_textures: HashMap::new(),
            user_global_state: state,
        }
    }

    pub fn connect(
        &mut self,
        label: &str,
        matrix: Matrix,
        img: &[u8],
        entity: Arc<Entity>,
        enabled: bool,
    ) {
        let texture = Texture {
            unloaded: img.to_vec(),
            entity: entity,
            matrix: matrix,
            dimensions: (500, 500),
            loaded: None,
            waiter: None,
        };
        match enabled {
            true => self
                .enabled_textures
                .insert(label.to_owned(), Arc::new(Mutex::new(texture))),
            false => self
                .disabled_textures
                .insert(label.to_owned(), Arc::new(Mutex::new(texture))),
        };
    }
}

pub struct VkSession {
    //instance: Arc<instance::Instance>,
    device: Arc<device::Device>,
    queue: Arc<device::Queue>,
    render_target: RenderTarget,
    render_pass: Arc<framebuffer::RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<framebuffer::FramebufferAbstract + Send + Sync>>,
    draw_pipeline: Arc<DrawGraphicsPipeline>,
}
pub type DrawGraphicsPipeline = pipeline::GraphicsPipeline<
    pipeline::vertex::SingleBufferDefinition<vertex::Vertex>,
    Box<vulkano::descriptor::PipelineLayoutAbstract + Send + Sync>,
    Arc<RenderPassAbstract + Send + Sync>,
>;
//pub type DrawGraphicsPipeline = Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>;

struct RenderTarget {
    event_loop: winit::EventsLoop,
    surface: Arc<swapchain::Surface<winit::Window>>,
    swapchain: Arc<swapchain::Swapchain<winit::Window>>,
    images: Vec<Arc<image::SwapchainImage<winit::Window>>>,
    dynamic_state: command_buffer::DynamicState,
}

impl VkSession {
    pub fn run<S>(game: Game<S>) -> Result<(), &'static str> {
        let instance = init::new_instance();

        println!("Listing discovered devices");
        let mut physical = None;
        for device in instance::PhysicalDevice::enumerate(&instance) {
            println!("{id}: {name}", id = device.index(), name = device.name());
            if device.index() == 0 {
                physical = Some(device);
            }
        }
        let physical = match physical {
            None => return Err("Unable to find an suitable graphics device"),
            Some(d) => d,
        };
        println!("Using {}", physical.name());

        let (surface, event_loop) = init::prepare_window(instance.clone());

        let queue_family = init::find_queue_family(&physical, &surface);

        let (device, mut queues) = init::setup_device(&physical, queue_family);

        let queue = queues.next().unwrap();

        let (swapchain, images) = init::swapchain(
            physical,
            surface.clone(),
            device.clone(),
            queue.clone(),
            VSYNC,
        );

        let render_pass = init::render_pass(device.clone(), swapchain.clone());

        let draw_pipeline = init::graphics_pipeline(device.clone(), render_pass.clone());

        let framebuffer = images
            .iter()
            .map(|image| {
                Arc::new(
                    framebuffer::Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<framebuffer::FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>();

        let viewport = {
            let size = surface.window().get_inner_size().unwrap();

            pipeline::viewport::Viewport {
                origin: [0.0, 0.0],
                dimensions: [size.width as f32, size.height as f32],
                depth_range: 0.0..1.0,
            }
        };

        let mut vk = VkSession {
            // instance: instance,
            device: device,
            queue: queue,
            render_target: RenderTarget {
                swapchain: swapchain,
                surface: surface,
                event_loop: event_loop,
                images: images,
                dynamic_state: command_buffer::DynamicState {
                    line_width: None,
                    viewports: Some(vec![viewport]),
                    scissors: None,
                },
            },
            render_pass: render_pass,
            framebuffers: framebuffer,
            draw_pipeline: draw_pipeline,
        };
        vk.recreate_dimensions_dependent().unwrap();
        vk.vk_main(game);

        Ok(())
    }

    pub fn recreate_dimensions_dependent(&mut self) -> Result<(), ()> {
        let window = self.render_target.surface.window();
        let dims: (u32, u32) = window
            .get_inner_size()
            .unwrap()
            .to_physical(window.get_hidpi_factor())
            .into();
        let new = self
            .render_target
            .swapchain
            .recreate_with_dimension([dims.0, dims.1])
            .unwrap_or_else(|e| {
                eprintln!("Resize failure, retrying once. ({})", e);
                let dims: (u32, u32) = window
                    .get_inner_size()
                    .unwrap()
                    .to_physical(window.get_hidpi_factor())
                    .into();
                self.render_target
                    .swapchain
                    .recreate_with_dimension([dims.0, dims.1])
                    .unwrap()
            });
        println!("{:?}", dims);

        self.render_target.swapchain = new.0;
        self.render_target.images = new.1;

        let viewport = pipeline::viewport::Viewport {
            origin: [0.0, 0.0],
            dimensions: [dims.0 as f32, dims.1 as f32],
            depth_range: 0.0..1.0,
        };
        let dynamic_state = command_buffer::DynamicState {
            line_width: None,
            viewports: Some(vec![viewport]),
            scissors: None,
        };
        self.render_target.dynamic_state = dynamic_state;

        self.framebuffers = self
            .render_target
            .images
            .iter()
            .map(|image| {
                Arc::new(
                    framebuffer::Framebuffer::start(self.render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<framebuffer::FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>();

        let vs = shader::vs::Shader::load(self.device.clone()).unwrap();
        let fs = shader::fs::Shader::load(self.device.clone()).unwrap();
        self.draw_pipeline = Arc::new(
            pipeline::GraphicsPipeline::start()
                .vertex_input_single_buffer::<vertex::Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_strip()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(framebuffer::Subpass::from(self.render_pass.clone(), 0).unwrap())
                .build(self.device.clone())
                .unwrap(),
        );

        Ok(())
    }
}
