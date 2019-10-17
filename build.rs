use std::path::Path;
use std::process::Command;

fn main() {
    let vert_input = Path::new("assets/shaders/vert_shader.vert");
    let vert_output = Path::new("assets/shaders/vert_shader.spv");
    let frag_input = Path::new("assets/shaders/frag_shader.frag");
    let frag_output = Path::new("assets/shaders/frag_shader.spv");

    compile_shader(vert_input, vert_output);
    compile_shader(frag_input, frag_output);
}

fn compile_shader(input: &Path, output: &Path) {
    Command::new("glslc")
        .args(&[input.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .status()
        .unwrap();
}
