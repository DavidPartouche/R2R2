use simplelog::{Config, LevelFilter, SimpleLogger};

use crate::render_manager::RenderManager;
use crate::window_manager::WindowManager;
use std::path::Path;
use vulkan_ray_tracing::glm;

pub struct ApplicationManager {
    render_manager: RenderManager,
    window_manager: Option<WindowManager>,
}

impl ApplicationManager {
    pub fn set_clear_color(&mut self, clear_color: &glm::Vec4) {
        self.render_manager.set_clear_color(clear_color);
    }

    pub fn load_scene(&mut self, filename: &Path) {
        self.render_manager.load_model(filename);
    }

    pub fn run(&mut self) {
        let window = self.window_manager.take();
        window
            .expect("Window already running, call run only once!")
            .run(|| {
                self.render_manager.draw();
            });
    }
}

pub struct ApplicationManagerBuilder {
    title: String,
    width: u32,
    height: u32,
}

impl Default for ApplicationManagerBuilder {
    fn default() -> Self {
        ApplicationManagerBuilder {
            title: String::from("R2R2"),
            width: 800,
            height: 600,
        }
    }
}

impl ApplicationManagerBuilder {
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

    pub fn build(self) -> ApplicationManager {
        SimpleLogger::init(LevelFilter::Trace, Config::default())
            .expect("Cannot create the logger!");

        let window = WindowManager::new(&self.title, self.width, self.height)
            .expect("Cannot create a window!");

        let size = window.size();
        let renderer = RenderManager::new(true, window.hwnd(), size.width, size.height);

        ApplicationManager {
            window_manager: Some(window),
            render_manager: renderer,
        }
    }
}
