use std::io::Cursor;
use std::{cell::RefCell, rc::Rc};

use glium::{implement_vertex, index::PrimitiveType, program, uniform, Frame, Surface};
use gtk::{glib, prelude::*, subclass::prelude::*};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

struct Renderer {
    context: Rc<glium::backend::Context>,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    opengl_texture: glium::texture::CompressedTexture2d,
    program: glium::Program,
}

impl Renderer {
    fn new(context: Rc<glium::backend::Context>) -> Self {
        // building a texture with "OpenGL" drawn on it
        let image = image::load(
            Cursor::new(&include_bytes!("../../opengl.png")[..]),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();
        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let opengl_texture = glium::texture::CompressedTexture2d::new(&context, image).unwrap();

        // building the vertex buffer, which contains all the vertices that we will draw
        let vertex_buffer = {
            glium::VertexBuffer::new(
                &context,
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
            glium::IndexBuffer::new(&context, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
                .unwrap();

        // compiling shaders and linking them together
        let program = program!(&context,
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

                        // Explicitly setting a color to the fragments works!
                        // So everything in the drawing pipeline seems to work,
                        // except sampling from the texture?
                        // f_color.r = 0.4;
                    }
                "
            },
        )
        .unwrap();

        Self {
            context,
            vertex_buffer,
            index_buffer,
            opengl_texture,
            program,
        }
    }

    fn draw(&self) {
        let mut frame = Frame::new(
            self.context.clone(),
            dbg!(self.context.get_framebuffer_dimensions()),
        );

        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: &self.opengl_texture
        };

        frame.clear_color(0.0, 0.0, 0.0, 0.5);
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

#[derive(Default)]
pub struct GliumGLArea {
    renderer: RefCell<Option<Renderer>>,
}

#[glib::object_subclass]
impl ObjectSubclass for GliumGLArea {
    const NAME: &'static str = "GliumGLArea";
    type Type = super::GliumGLArea;
    type ParentType = gtk::GLArea;
}

impl ObjectImpl for GliumGLArea {}

impl WidgetImpl for GliumGLArea {
    fn realize(&self) {
        self.parent_realize();

        let widget = self.obj();
        if widget.error().is_some() {
            return;
        }

        // SAFETY: we know the GdkGLContext exists as we checked for errors above, and
        // we haven't done any operations on it which could lead to glium's
        // state mismatch. (In theory, GTK doesn't do any state-breaking
        // operations on the context either.)
        //
        // We will also ensure glium's context does not outlive the GdkGLContext by
        // destroying it in `unrealize()`.
        let context = unsafe {
            glium::backend::Context::new(
                widget.clone(),
                true,
                glium::debug::DebugCallbackBehavior::PrintAll,
            )
        }
        .unwrap();
        *self.renderer.borrow_mut() = Some(Renderer::new(context));
    }

    fn unrealize(&self) {
        *self.renderer.borrow_mut() = None;

        self.parent_unrealize();
    }
}

impl GLAreaImpl for GliumGLArea {
    fn render(&self, _context: &gtk::gdk::GLContext) -> glib::Propagation {
        self.renderer.borrow().as_ref().unwrap().draw();

        glib::Propagation::Stop
    }
}
