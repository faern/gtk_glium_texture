use std::{cell::RefCell, rc::Rc};

use glium::Frame;
use gtk::{glib, prelude::*, subclass::prelude::*};

struct Renderer {
    context: Rc<glium::backend::Context>,
    draw_texture: crate::gl::DrawTexture,
}

impl Renderer {
    fn new(context: Rc<glium::backend::Context>) -> Self {
        let draw_texture = crate::gl::DrawTexture::new(&context);
        Self {
            context,
            draw_texture,
        }
    }

    fn draw(&self) {
        let frame = Frame::new(
            self.context.clone(),
            self.context.get_framebuffer_dimensions(),
        );
        self.draw_texture.draw(frame);
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
