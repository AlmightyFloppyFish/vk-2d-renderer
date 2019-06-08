use crate::renderer::entity::Texture;
use crate::renderer::vertex::Vertex;
use crate::renderer::VkSession;
use std::sync::{Arc, Mutex};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device;
use vulkano::sampler::{BorderColor, Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::swapchain;
use vulkano::sync::{now, GpuFuture};

pub type DrawBuffer = Vec<Arc<Mutex<Texture>>>;

impl VkSession {
    // Prints draw buffer to swapchain
    // Needs take mut self to update swapchain if required.
    pub fn present(
        &mut self,
        draw_buffer: &mut DrawBuffer,
        mut prev_frame: Box<GpuFuture + Send + Sync>,
    ) -> Box<GpuFuture + Send + Sync> {
        let (buffer_num, gpu_fut) =
            match swapchain::acquire_next_image(self.render_target.swapchain.clone(), None) {
                Err(e) => {
                    eprintln!("Recreating swapchain because {}", e);
                    self.recreate_dimensions_dependent().unwrap();
                    return self.present(draw_buffer, prev_frame);
                }
                Ok(out) => out,
            };
        prev_frame.cleanup_finished();
        //prev_frame = Box::new(now(self.device.clone()));

        let mut command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        .begin_render_pass(
            self.framebuffers[buffer_num].clone(),
            false,
            vec![[0.0, 0.0, 0.0, 1.0].into()],
        )
        .unwrap();

        let mut prev_frame = Box::new(prev_frame.join(gpu_fut)) as Box<GpuFuture + Sync + Send>;
        for i in 0..draw_buffer.len() {
            let mut draw_set = draw_buffer[i].lock().unwrap();
            if draw_set.waiter.is_some() {
                prev_frame = Box::new(prev_frame.join(Box::new(draw_set.waiter.take().unwrap())));
            }

            let window = self.render_target.surface.window();

            command_buffer = command_buffer
                .draw(
                    self.draw_pipeline.clone(),
                    &self.render_target.dynamic_state,
                    CpuAccessibleBuffer::<[Vertex]>::from_iter(
                        self.device.clone(),
                        BufferUsage::all(),
                        draw_set
                            .to_vert(
                                window
                                    .get_inner_size()
                                    .unwrap()
                                    .to_physical(window.get_hidpi_factor())
                                    .into(),
                            )
                            .iter()
                            .cloned(),
                    )
                    .unwrap(),
                    draw_set.loaded.clone().unwrap(),
                    (),
                )
                .unwrap()
        }

        let cb = command_buffer
            .end_render_pass()
            .map_err(|e| eprintln!("\n\n{:?}\n\n", e))
            .unwrap()
            .build()
            .map_err(|e| eprintln!("\n\n{:?}\n\n", e))
            .unwrap();

        let f = match (Box::new(prev_frame) as Box<GpuFuture + Send + Sync>)
            .then_execute(self.queue.clone(), cb)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                self.render_target.swapchain.clone(),
                buffer_num,
            )
            .then_signal_fence_and_flush()
        {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Swapchain does not match. Updating swapchain ({})", e);
                self.recreate_dimensions_dependent().unwrap();
                return self.present(
                    draw_buffer,
                    Box::new(now(self.device.clone())) as Box<GpuFuture + Send + Sync>,
                );
            }
        };
        Box::new(f) as Box<GpuFuture + Send + Sync>
    }
}

pub fn default_sampler(device: Arc<device::Device>) -> Arc<Sampler> {
    Sampler::new(
        device,
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        //SamplerAddressMode::ClampToBorder(BorderColor::FloatTransparentBlack),
        //SamplerAddressMode::ClampToBorder(BorderColor::FloatTransparentBlack),
        //SamplerAddressMode::ClampToBorder(BorderColor::FloatTransparentBlack),
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0,
        1.0,
        0.0,
        0.0,
    )
    .unwrap()
}
