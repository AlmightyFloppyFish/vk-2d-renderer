mod renderer;

use renderer::entity::Matrix;
use renderer::*;
use std::sync::Arc;

struct TestEntity {
    hp: u64,
}

impl renderer::entity::Entity for TestEntity {
    fn init(&mut self) {
        println!("TestEntity init ({})", self.hp);
    }
    fn update(&mut self) {
        println!("TestEntity update");
    }
}

fn main() {
    let mut game = Game::new(());

    game.connect(
        "test",
        Matrix::new((0.5, 0.5), (0.5, 0.5)),
        include_bytes!("test.png"),
        Arc::new(TestEntity { hp: 100 }),
        true,
    );

    let _vk = VkSession::run(game).unwrap();
}
