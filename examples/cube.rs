use std::path::Path;

use r2r2::application::ApplicationBuilder;
use vulkan_helpers::glm;

fn main() {
    let mut app = ApplicationBuilder::new().build();
    app.renderer.set_clear_value(glm::vec4(1.0, 1.0, 1.0, 1.0));
    app.renderer.load_model(Path::new("assets/models/cube.obj"));
    app.run();
}
