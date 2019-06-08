pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 scale;
layout(location = 0) out vec2 tex_coords;

vec2 to_vk_numbers(vec2 n) {
    return (n * 2 ) - vec2(1.0);
}

void main() {
    vec2 n = to_vk_numbers(position);
    gl_Position = vec4(n, 0.0, 1.0);

    tex_coords = to_vk_numbers(n);
}"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    f_color = texture(
        tex, tex_coords
    );
}
"
    }
}
