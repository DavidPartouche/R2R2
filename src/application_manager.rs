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
    pub fn run(&mut self) {
        let window = self.window_manager.take();
        window
            .expect("Window already running, call run only once!")
            .run(|| {
                self.render_manager.update_camera();
                self.render_manager.render_scene();
            });
    }
}

pub struct ApplicationManagerBuilder {
    title: String,
    width: u32,
    height: u32,
    scene: String,
    clear_color: glm::Vec4,
}

impl Default for ApplicationManagerBuilder {
    fn default() -> Self {
        ApplicationManagerBuilder {
            title: String::from("R2R2"),
            width: 800,
            height: 600,
            scene: String::new(),
            clear_color: glm::vec4(0.0, 0.0, 0.0, 1.0),
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

    pub fn with_clear_color(mut self, clear_color: glm::Vec4) -> Self {
        self.clear_color = clear_color;
        self
    }

    pub fn with_scene(mut self, scene: &str) -> Self {
        self.scene = scene.to_string();
        self
    }

    pub fn build(self) -> ApplicationManager {
        SimpleLogger::init(LevelFilter::Trace, Config::default())
            .expect("Cannot create the logger!");

        let window = WindowManager::new(&self.title, self.width, self.height)
            .expect("Cannot create a window!");

        let size = window.size();
        let mut render_manager = RenderManager::new(true, window.hwnd(), size.width, size.height);

        render_manager.set_clear_color(self.clear_color);

        let scene = Path::new(&self.scene);
        if !scene.exists() {
            panic!("No scene loaded");
        }
        render_manager.load_model(scene);

        ApplicationManager {
            window_manager: Some(window),
            render_manager,
        }
    }
}
