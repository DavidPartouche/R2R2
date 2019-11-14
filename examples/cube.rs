use r2r2::application_manager::ApplicationManagerBuilder;

fn main() {
    let mut app = ApplicationManagerBuilder::new()
        .with_width(800)
        .with_height(600)
        .with_scene("assets/models/cube.obj")
        .build();

    app.run();
}
