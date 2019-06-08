use crate::renderer::{Game, VkSession};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use vulkano::sync;

pub(crate) mod draw;
mod framecounter;
use framecounter::FPSCounter;

const TODO_THREAD_COUNT: i32 = 4;

impl VkSession {
    pub fn vk_main<S>(mut self, mut game: Game<S>) {
        // I think I'll do similar design to zircon 1.0, with one user update loop, and one render
        // update loop.

        // One cool difference i could make is giving the user "Game" in closures
        // I should also consider running all user updates concurrently

        let mut draw_buffer = draw::DrawBuffer::new();

        // TODO: Make concurrent
        for t in game.enabled_textures.values_mut() {
            t.lock().unwrap().load_gpu(
                self.queue.clone(),
                self.device.clone(),
                self.draw_pipeline.clone(),
            );
        }
        for t in game.disabled_textures.values_mut() {
            t.lock().unwrap().load_gpu(
                self.queue.clone(),
                self.device.clone(),
                self.draw_pipeline.clone(),
            );
        }

        let shared_state = Arc::new(Mutex::new(game));

        // TODO: Mutation settings can be in Arc<Texture>

        let user_state = shared_state.clone();
        thread::spawn(move || {
            loop {
                // TODO: Better timer that takes computation time into consideration
                thread::sleep(Duration::from_millis(60));
            }
        });

        let mut fps = FPSCounter::new();

        let mut prev_frame =
            Box::new(sync::now(self.device.clone())) as Box<sync::GpuFuture + Send + Sync>;
        loop {
            // Prepare all textures that'll be rendered
            let mut game = shared_state.lock().unwrap();
            for (_k, t) in game.enabled_textures.drain() {
                draw_buffer.push(t.clone());
            }
            drop(game);
            prev_frame = self.present(&mut draw_buffer, prev_frame);
            fps.tick_and_display();
        }
    }
}
