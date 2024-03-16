use std::io::Cursor;

use glium::backend::Facade;
use glium::program::Program;
use glium::texture::{CompressedTexture2d, RawImage2d};
use glium::{implement_vertex, index::PrimitiveType, program};
use glium::{IndexBuffer, VertexBuffer};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub fn compile_program<F: Facade>(context: &F) -> Program {
    program!(context,
        140 => {
            vertex: "
                #version 140

                uniform mat4 matrix;
                in vec2 position;
                in vec2 tex_coords;

                out vec2 v_tex_coords;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 140

                uniform sampler2D tex;
                in vec2 v_tex_coords;

                out vec4 f_color;

                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            "
        },
    )
    .unwrap()
}

pub fn rectangle_vertices<F: Facade>(context: &F) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let vertex_buffer = {
        glium::VertexBuffer::new(
            context,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
        )
        .unwrap()
    };

    // building the index buffer
    let index_buffer =
        glium::IndexBuffer::new(context, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
            .unwrap();

    (vertex_buffer, index_buffer)
}

pub fn load_texture<F: Facade>(context: &F) -> CompressedTexture2d {
    // building a texture with "OpenGL" drawn on it
    let image = image::load(
        Cursor::new(&include_bytes!("../opengl.png")[..]),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();

    let image_dimensions = image.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    CompressedTexture2d::new(context, image).unwrap()
}
