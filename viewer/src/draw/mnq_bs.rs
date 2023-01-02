use macroquad::{
    miniquad as mnq,
    prelude::{vec2, Vertex, WHITE},
};

/// Vertex shader taking advantage of instancing.
/// https://learnopengl.com/Advanced-OpenGL/Instancing
const VERT: &str = r##"
#version 100

attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}    
"##;

const FRAG: &str = r##"
#version 100

varying lowp vec4 color;
uniform sampler2D Texture;

void main() {
    gl_FragColor = color;
}
"##;

fn build_bindings(&self, ctx: &mut miniquad::Context, pos_vert_buf: Buffer) -> mnq::Bindings {
    let mut verts = vec![Vertex::new(0.0, 0.0, 1.0, 0.0, 0.0, WHITE)];
    // TIL i cannot spell "twelfthfs"
    for twelfths in [1, 3, 5, 7, 9, 11] {
        let angle = TAU * twelfths as f32 / 12.0;
        let (y, x) = angle.sin_cos();
        verts.push(vec2(x, y));
    }

    let vert_buf = Buffer::immutable(ctx, BufferType::VertexBuffer, &verts);

    #[rustfmt::skip]
    let idxs = [
        0, 1, 2,
        0, 2, 3,
        0, 3, 4,
        0, 4, 5,
        0, 5, 6,
        0, 6, 1,
    ];
    let idx_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &idxs);

    mnq::Bindings {
        vertex_buffers: vec![vert_buf],
        index_buffer: vec![idx_buffer],
        images: vec![],
    }
}
