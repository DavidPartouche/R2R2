use simplelog::{Config, LevelFilter, SimpleLogger};

use crate::renderer::Renderer;
use crate::window::Window;

pub struct Application {
    pub renderer: Renderer,
    window: Option<Window>,
}

impl Application {
    pub fn run(&mut self) {
        let window = self.window.take();
        window
            .expect("Window already running, call run only once!")
            .run(|| {
                self.renderer.draw_frame();
            });
    }
}

pub struct ApplicationBuilder {
    title: String,
    width: u32,
    height: u32,
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        ApplicationBuilder {
            title: String::from("R2R2"),
            width: 800,
            height: 600,
        }
    }
}

impl ApplicationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn build(self) -> Application {
        SimpleLogger::init(LevelFilter::Trace, Config::default())
            .expect("Cannot create the logger!");

        let window =
            Window::new(&self.title, self.width, self.height).expect("Cannot create a window!");

        let renderer = Renderer::new(true, window.hwnd(), self.width, self.height);

        Application {
            window: Some(window),
            renderer,
        }
    }
}
