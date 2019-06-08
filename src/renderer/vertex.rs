#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub scale: (f32, f32),
}
vulkano::impl_vertex!(Vertex, position, scale);

impl Vertex {
    pub fn square(pos: (f32, f32), size: (f32, f32)) -> [Vertex; 4] {
        [
            Vertex {
                // Top-Left
                position: [pos.0, pos.1],
                scale: (1.0, 1.0),
            },
            Vertex {
                // Bottom-Left
                position: [pos.0, pos.1 + size.1],
                scale: (1.0, 1.0),
            },
            Vertex {
                // Top-Right
                position: [pos.0 + size.0, pos.1],
                scale: (1.0, 1.0),
            },
            Vertex {
                // Bottom-Right
                position: [pos.0 + size.0, pos.1 + size.1],
                scale: (1.0, 1.0),
            },
        ]
    }
}
