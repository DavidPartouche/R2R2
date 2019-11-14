use std::path::Path;

use r2r2::application_manager::ApplicationManagerBuilder;

fn main() {
    let mut app = ApplicationManagerBuilder::new()
        .with_width(800)
        .with_height(600)
        .build();
    app.load_scene(Path::new("assets/models/cube.obj"));
    app.run();
}
