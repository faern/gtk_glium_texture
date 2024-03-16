use std::ptr;

use gtk::{glib, prelude::*};

mod glium_gl_area;
use glium_gl_area::GliumGLArea;

pub mod gl;
mod glium;

fn main() -> glib::ExitCode {
    // Load GL pointers from epoxy (GL context management library used by GTK).
    {
        #[cfg(target_os = "macos")]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.0.dylib") }.unwrap();
        #[cfg(all(unix, not(target_os = "macos")))]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
        #[cfg(windows)]
        let library = libloading::os::windows::Library::open_already_loaded("libepoxy-0.dll")
            .or_else(|_| libloading::os::windows::Library::open_already_loaded("epoxy-0.dll"))
            .unwrap();

        epoxy::load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(ptr::null())
        });
    }

    // Spawn glium window (that works and draws texture!) in a separate thread
    std::thread::spawn(move || {
        glium::State::run_loop();
    });

    // Run GTK window (where textures don't render!) in main thread
    let application = gtk::Application::builder()
        .application_id("com.example.not-working-gtk-textures")
        .build();
    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Glium in GLArea"));

    let widget = GliumGLArea::default();
    window.set_child(Some(&widget));

    window.present();
}
