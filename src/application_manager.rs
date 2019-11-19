use simplelog::{Config, LevelFilter, SimpleLogger};

use crate::camera_manager::{CameraManager, CameraProperties};
use crate::input_manager::InputManager;
use crate::render_manager::RenderManager;
use crate::scene::scene_manager::SceneManager;
use crate::window_manager::WindowManager;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use vulkan_ray_tracing::glm;

pub struct ApplicationManager {
    window_manager: Option<WindowManager>,
    scene_manager: SceneManager,
    input_manager: Rc<RefCell<InputManager>>,
    camera_manager: Rc<RefCell<CameraManager>>,
    render_manager: Rc<RefCell<RenderManager>>,
    target_framerate: u32,
    begin_ticks: Instant,
    delta_time: f32,
}

impl ApplicationManager {
    pub fn load_default_scene(&mut self) {
        self.scene_manager.load_default_scene();
    }

    pub fn run(&mut self) {
        let window = self.window_manager.take();
        window
            .expect("Window already running, call run only once!")
            .run(|window, mouse_position, events| {
                self.input_manager.borrow_mut().update(events);
                self.camera_manager
                    .borrow_mut()
                    .update(window, mouse_position, self.delta_time);
                self.render_manager.borrow_mut().render_scene();
                let end_ticks = Instant::now();
                self.delta_time = end_ticks.duration_since(self.begin_ticks).as_secs_f32();
                // If delta time is too big, it probably means that we hit a breakpoint
                if self.delta_time > 1.0 {
                    self.delta_time = 1.0 / self.target_framerate as f32;
                }

                self.begin_ticks = end_ticks;
            });
    }
}

pub struct ApplicationManagerBuilder {
    title: String,
    width: u32,
    height: u32,
    scene: String,
    clear_color: glm::Vec4,
    target_framerate: u32,
    camera_properties: CameraProperties,
}

impl Default for ApplicationManagerBuilder {
    fn default() -> Self {
        ApplicationManagerBuilder {
            title: String::from("R2R2"),
            width: 800,
            height: 600,
            scene: String::new(),
            clear_color: glm::vec4(0.0, 0.0, 0.0, 1.0),
            target_framerate: 60,
            camera_properties: CameraProperties::default(),
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

    pub fn with_target_framerate(mut self, target_framerate: u32) -> Self {
        self.target_framerate = target_framerate;
        self
    }

    pub fn with_camera(mut self, camera_properties: CameraProperties) -> Self {
        self.camera_properties = camera_properties;
        self
    }

    pub fn build(self) -> ApplicationManager {
        SimpleLogger::init(LevelFilter::Trace, Config::default())
            .expect("Cannot create the logger!");

        let window = WindowManager::new(&self.title, self.width, self.height)
            .expect("Cannot create a window!");

        let input_manager = Rc::new(RefCell::new(InputManager::new()));

        let camera_manager = Rc::new(RefCell::new(CameraManager::new(
            Rc::clone(&input_manager),
            self.width as f32,
            self.height as f32,
            self.camera_properties,
        )));

        let size = window.size();
        let render_manager = Rc::new(RefCell::new(RenderManager::new(
            true,
            window.hwnd(),
            size.width,
            size.height,
            Rc::clone(&camera_manager),
        )));
        render_manager.borrow().set_clear_color(self.clear_color);

        let scene_manager = SceneManager::new(&self.scene, Rc::clone(&render_manager));

        ApplicationManager {
            window_manager: Some(window),
            scene_manager,
            input_manager,
            camera_manager,
            render_manager,
            target_framerate: self.target_framerate,
            begin_ticks: Instant::now(),
            delta_time: 1.0 / self.target_framerate as f32,
        }
    }
}
