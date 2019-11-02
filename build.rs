use std::path::Path;
use std::process::Command;

fn main() {
    let closesthit_input = Path::new("assets/shaders/closesthit.rchit");
    let closesthit_output = Path::new("assets/shaders/closesthit.spv");
    compile_shader(closesthit_input, closesthit_output);

    let miss_input = Path::new("assets/shaders/miss.rmiss");
    let miss_output = Path::new("assets/shaders/miss.spv");
    compile_shader(miss_input, miss_output);

    let raygen_input = Path::new("assets/shaders/raygen.rgen");
    let raygen_output = Path::new("assets/shaders/raygen.spv");
    compile_shader(raygen_input, raygen_output);
}

fn compile_shader(input: &Path, output: &Path) {
    Command::new("glslc")
        .args(&[input.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .status()
        .unwrap();
}
