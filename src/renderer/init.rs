use std::sync::Arc;

use crate::renderer::vertex::Vertex;
use crate::renderer::{shader, DrawGraphicsPipeline};
use vulkano::device;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer;
use vulkano::instance;
use vulkano::pipeline;
use vulkano::swapchain;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;

pub fn new_instance() -> Arc<instance::Instance> {
    instance::Instance::new(None, &vulkano_win::required_extensions(), None).unwrap()
}

pub fn prepare_window(
    instance: Arc<instance::Instance>,
) -> (Arc<swapchain::Surface<winit::Window>>, winit::EventsLoop) {
    let events_loop = winit::EventsLoop::new();
    (
        winit::WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap(),
        events_loop,
    )
}

pub fn find_queue_family<'a>(
    physical: &'a instance::PhysicalDevice,
    surface: &swapchain::Surface<winit::Window>,
) -> instance::QueueFamily<'a> {
    physical
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .unwrap()
}

pub fn setup_device(
    physical: &instance::PhysicalDevice,
    queue_family: instance::QueueFamily,
) -> (Arc<Device>, device::QueuesIter) {
    Device::new(
        *physical,
        physical.supported_features(),
        &DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        },
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap()
}

pub fn swapchain(
    physical: instance::PhysicalDevice,
    surface: Arc<Surface<winit::Window>>,
    device: Arc<device::Device>,
    queue: Arc<device::Queue>,
    vsync: bool,
) -> (
    Arc<swapchain::Swapchain<winit::Window>>,
    Vec<Arc<vulkano::image::swapchain::SwapchainImage<winit::Window>>>,
) {
    // Return window, borrow surface
    let caps = surface.capabilities(physical).unwrap();
    let usage = caps.supported_usage_flags;

    let alpha = caps.supported_composite_alpha.iter().next().unwrap();

    let format = caps.supported_formats[0].0;

    let window = surface.window();

    let initial_dimensions = if let Some(dimensions) = window.get_inner_size() {
        let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
        [dimensions.0, dimensions.1]
    } else {
        panic!("Window didn't exist when it was expected to");
    };

    swapchain::Swapchain::new(
        device,
        surface.clone(),
        caps.min_image_count,
        format,
        initial_dimensions,
        1,
        usage,
        &queue,
        swapchain::SurfaceTransform::Identity,
        alpha,
        if vsync {
            swapchain::PresentMode::Fifo
        } else {
            swapchain::PresentMode::Immediate
        },
        true,
        None,
    )
    .unwrap()
}

pub fn render_pass(
    device: Arc<device::Device>,
    swapchain: Arc<swapchain::Swapchain<winit::Window>>,
) -> Arc<framebuffer::RenderPassAbstract + Send + Sync> {
    Arc::new(
        vulkano::single_pass_renderpass!(
            device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap(),
    ) as Arc<framebuffer::RenderPassAbstract + Send + Sync>
}

pub fn graphics_pipeline(
    device: Arc<device::Device>,
    render_pass: Arc<framebuffer::RenderPassAbstract + Send + Sync>,
) -> Arc<DrawGraphicsPipeline> {
    let vs = shader::vs::Shader::load(device.clone()).unwrap();
    let fs = shader::fs::Shader::load(device.clone()).unwrap();
    Arc::new(
        pipeline::GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_strip()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(framebuffer::Subpass::from(render_pass, 0).unwrap())
            .build(device)
            .unwrap(),
    )
}
