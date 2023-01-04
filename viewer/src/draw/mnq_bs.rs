use std::f32::consts::TAU;

use macroquad::{
    miniquad::{
        self as mnq, Buffer, BufferLayout, BufferType, Pipeline, Shader, VertexAttribute,
        VertexFormat, VertexStep,
    },
    prelude::{vec2, PipelineParams, Vec2, Vec4, Vertex, WHITE},
    window::InternalGlContext,
};

const MAX_HEXES: usize = 1_000;

pub struct FiddlyMiniquadBullshit {
    pipeline: mnq::Pipeline,
    bindings: mnq::Bindings,

    hexes: Vec<GpuHex>,
}

impl FiddlyMiniquadBullshit {
    pub fn make() -> Self {
        let InternalGlContext {
            quad_context: context,
            ..
        } = unsafe { macroquad::prelude::get_internal_gl() };

        let bindings = build_bindings(context);
        let shader = Shader::new(context, shader::VERT, shader::FRAG, shader::meta()).unwrap();

        let pipeline = Pipeline::with_params(
            context,
            &[
                // Hexagon vertices
                BufferLayout::default(),
                // Packed data verts
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                // mandatory vertex attributes
                VertexAttribute::with_buffer("in_attr_pos", VertexFormat::Float3, 0),
                VertexAttribute::with_buffer("in_attr_uv", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("in_attr_color", VertexFormat::Float4, 0),
                // these reflect the structure of the gpu particle
                VertexAttribute::with_buffer("instPos", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("instPackedState", VertexFormat::Int1, 1),
            ],
            shader,
            PipelineParams {
                color_blend: None,
                ..Default::default()
            },
        );

        Self {
            pipeline,
            bindings,
            hexes: Vec::new(),
        }
    }
}

#[repr(C)]
struct GpuHex {
    pos: Vec2,
    data: u32,
}

fn build_bindings(ctx: &mut mnq::Context) -> mnq::Bindings {
    let hex_data_buf = Buffer::stream(
        ctx,
        BufferType::VertexBuffer,
        MAX_HEXES * std::mem::size_of::<GpuHex>(),
    );

    let mut verts = Vec::new();
    // TIL i cannot spell "twelfthfs"
    for twelfths in [1, 3, 5, 7, 9, 11] {
        let angle = TAU * twelfths as f32 / 12.0;
        let (y, x) = angle.sin_cos();
        verts.push(Vertex::new(x, y, 1.0, 0.0, 0.0, WHITE));
    }

    let vert_buf = Buffer::immutable(ctx, BufferType::VertexBuffer, &verts);

    #[rustfmt::skip]
    let idxs = [
        0, 1, 2,
        0, 2, 5,
        2, 5, 4,
        2, 4, 3,
    ];
    let idx_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &idxs);

    mnq::Bindings {
        vertex_buffers: vec![vert_buf, hex_data_buf],
        index_buffer: idx_buffer,
        images: vec![],
    }
}

mod shader {
    use macroquad::{
        miniquad::{ShaderMeta, UniformBlockLayout, UniformDesc},
        prelude::UniformType,
    };

    /// Vertex shader taking advantage of instancing.
    /// https://learnopengl.com/Advanced-OpenGL/Instancing
    pub const VERT: &str = r##"
#version 100

attribute vec2 instPos;
attribute uint instPackedState;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0 / 255.0;
    uv = texcoord;
}    
"##;

    pub const FRAG: &str = r##"
#version 100

void main() {
    gl_FragColor = color;
}
"##;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: Vec::new(),
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("Model", UniformType::Mat4),
                    UniformDesc::new("Projection", UniformType::Mat4),
                ],
            },
        }
    }
}
