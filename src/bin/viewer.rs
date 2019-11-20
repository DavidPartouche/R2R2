use clap::{App, Arg};
use r2r2::application_manager::ApplicationManagerBuilder;
use vulkan_ray_tracing::glm;

fn main() {
    let matches = App::new("r2r2-viewer")
        .arg(Arg::with_name("scene").takes_value(true).required(true))
        .get_matches();

    let scene_file = matches.value_of("scene").unwrap();

    let mut app = ApplicationManagerBuilder::new()
        .with_width(800)
        .with_height(600)
        .with_clear_color(glm::vec4(1.0, 1.0, 1.0, 1.0))
        .with_scene(scene_file)
        .build();

    app.load_default_scene();
    app.run();
}
