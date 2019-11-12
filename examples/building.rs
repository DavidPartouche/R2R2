use std::path::Path;

use r2r2::application::ApplicationBuilder;

fn main() {
    let mut app = ApplicationBuilder::new()
        .with_width(800)
        .with_height(600)
        .build();
    app.renderer
        .load_model(Path::new("assets/models/Medieval_building.obj"));
    app.run();
}
