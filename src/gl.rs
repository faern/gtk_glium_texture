use std::io::Cursor;

use glium::backend::Facade;
use glium::program::Program;
use glium::texture::{CompressedTexture2d, RawImage2d};
use glium::uniform;
use glium::Surface;
use glium::{implement_vertex, index::PrimitiveType, program};
use glium::{Frame, IndexBuffer, VertexBuffer};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

fn compile_program<F: Facade>(context: &F) -> Program {
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

fn rectangle_vertices<F: Facade>(context: &F) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
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

    let index_buffer =
        glium::IndexBuffer::new(context, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
            .unwrap();

    (vertex_buffer, index_buffer)
}

fn load_texture<F: Facade>(context: &F) -> CompressedTexture2d {
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

pub struct DrawTexture {
    vertex_buffer: glium::VertexBuffer<crate::gl::Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    opengl_texture: glium::texture::CompressedTexture2d,
    program: glium::Program,
}

impl DrawTexture {
    pub fn new<F: Facade>(context: &F) -> Self {
        let opengl_texture = load_texture(context);
        let (vertex_buffer, index_buffer) = rectangle_vertices(context);
        let program = compile_program(context);

        Self {
            vertex_buffer,
            index_buffer,
            opengl_texture,
            program,
        }
    }

    pub fn draw(&self, mut frame: Frame) {
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: &self.opengl_texture
        };

        frame.clear_color(0.0, 0.0, 0.0, 0.0);
        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        frame.finish().unwrap();
    }
}
