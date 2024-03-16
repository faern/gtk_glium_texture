use std::{cell::RefCell, rc::Rc};

use glium::{uniform, Frame, Surface};
use gtk::{glib, prelude::*, subclass::prelude::*};

struct Renderer {
    context: Rc<glium::backend::Context>,
    vertex_buffer: glium::VertexBuffer<crate::gl::Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    opengl_texture: glium::texture::CompressedTexture2d,
    program: glium::Program,
}

impl Renderer {
    fn new(context: Rc<glium::backend::Context>) -> Self {
        let opengl_texture = crate::gl::load_texture(&context);
        let (vertex_buffer, index_buffer) = crate::gl::rectangle_vertices(&context);
        let program = crate::gl::compile_program(&context);

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
