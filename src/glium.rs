use glium::Display;
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::WindowSurface;
use raw_window_handle::HasRawWindowHandle;
use std::num::NonZeroU32;
use winit::platform::wayland::EventLoopBuilderExtWayland;

pub struct Application {
    draw_texture: crate::gl::DrawTexture,
}

impl Application {
    fn new(display: &Display<WindowSurface>) -> Self {
        Self {
            draw_texture: crate::gl::DrawTexture::new(display),
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let frame = display.draw();
        self.draw_texture.draw(frame);
    }
}

pub struct State {
    pub display: glium::Display<WindowSurface>,
    pub window: winit::window::Window,
    pub context: Application,
}

impl State {
    pub fn new<W>(event_loop: &winit::event_loop::EventLoopWindowTarget<W>, visible: bool) -> Self {
        let window_builder = winit::window::WindowBuilder::new()
            .with_title("Glium image example")
            .with_visible(visible);
        let config_template_builder = glutin::config::ConfigTemplateBuilder::new();
        let display_builder =
            glutin_winit::DisplayBuilder::new().with_window_builder(Some(window_builder));

        // First we create a window
        let (window, gl_config) = display_builder
            .build(event_loop, config_template_builder, |mut configs| {
                // Just use the first configuration since we don't have any special preferences here
                configs.next().unwrap()
            })
            .unwrap();
        let window = window.unwrap();

        // Then the configuration which decides which OpenGL version we'll end up using, here we just use the default which is currently 3.3 core
        // When this fails we'll try and create an ES context, this is mainly used on mobile devices or various ARM SBC's
        // If you depend on features available in modern OpenGL Versions you need to request a specific, modern, version. Otherwise things will very likely fail.
        let raw_window_handle = window.raw_window_handle();
        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(Some(raw_window_handle));
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(Some(raw_window_handle));

        let not_current_gl_context = Some(unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        });

        // Determine our framebuffer size based on the window size, or default to 800x600 if it's invisible
        let (width, height): (u32, u32) = if visible {
            window.inner_size().into()
        } else {
            (800, 600)
        };
        let attrs = glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
        // Now we can create our surface, use it to make our context current and finally create our display
        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };
        let current_context = not_current_gl_context
            .unwrap()
            .make_current(&surface)
            .unwrap();
        let display = glium::Display::from_context_surface(current_context, surface).unwrap();

        Self::from_display_window(display, window)
    }

    pub fn from_display_window(
        display: glium::Display<WindowSurface>,
        window: winit::window::Window,
    ) -> Self {
        let context = Application::new(&display);
        Self {
            display,
            window,
            context,
        }
    }

    /// Start the event_loop and keep rendering frames until the program is closed
    pub fn run_loop() {
        let event_loop = winit::event_loop::EventLoopBuilder::new()
            .with_any_thread(true)
            .build()
            .expect("event loop building");
        let mut state: Option<State> = None;

        let result = event_loop.run(move |event, window_target| {
            match event {
                // The Resumed/Suspended events are mostly for Android compatiblity since the context can get lost there at any point.
                // For convenience's sake the Resumed event is also delivered on other platforms on program startup.
                winit::event::Event::Resumed => {
                    state = Some(State::new(window_target, true));
                }
                winit::event::Event::Suspended => state = None,
                // By requesting a redraw in response to a AboutToWait event we get continuous rendering.
                // For applications that only change due to user input you could remove this handler.
                winit::event::Event::AboutToWait => {
                    if let Some(state) = &state {
                        state.window.request_redraw();
                    }
                }
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::Resized(new_size) => {
                        if let Some(state) = &state {
                            state.display.resize(new_size.into());
                        }
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        if let Some(state) = &mut state {
                            state.context.draw_frame(&state.display);
                        }
                    }
                    // Exit the event loop when requested (by closing the window for example) or when
                    // pressing the Esc key.
                    winit::event::WindowEvent::CloseRequested
                    | winit::event::WindowEvent::KeyboardInput {
                        event:
                            winit::event::KeyEvent {
                                state: winit::event::ElementState::Pressed,
                                logical_key:
                                    winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                                ..
                            },
                        ..
                    } => window_target.exit(),
                    // Every other event
                    _ev => {}
                },
                _ => (),
            };
        });
        result.unwrap();
    }
}
