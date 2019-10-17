use nalgebra_glm as glm;

use r2r2::application::ApplicationBuilder;

fn main() {
    let mut app = ApplicationBuilder::new().build();
    app.renderer.set_clear_value(glm::vec4(1.0, 0.0, 1.0, 1.0));
    app.run();
}
