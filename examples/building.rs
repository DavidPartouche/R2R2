use r2r2::application_manager::ApplicationManagerBuilder;
use vulkan_ray_tracing::glm;

fn main() {
    let mut app = ApplicationManagerBuilder::new()
        .with_width(800)
        .with_height(600)
        .with_clear_color(glm::vec4(0.3, 0.3, 0.3, 0.0))
        .with_scene("assets/models/Medieval_building.obj")
        .build();

    app.run();
}
