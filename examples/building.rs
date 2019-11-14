use std::path::Path;

use r2r2::application::ApplicationBuilder;
use vulkan_ray_tracing::glm;

fn main() {
    let mut app = ApplicationBuilder::new()
        .with_width(800)
        .with_height(600)
        .build();
    app.renderer.set_clear_color(glm::vec4(0.3, 0.3, 0.3, 0.0));
    app.renderer
        .load_model(Path::new("assets/models/Medieval_building.obj"));
    app.run();
}
