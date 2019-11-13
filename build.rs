use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let shader_files = std::fs::read_dir(Path::new("assets/shaders/")).unwrap();

    for shader_file in shader_files {
        let input = shader_file.unwrap().path();
        if let Some(extension) = input.extension() {
            if extension.eq("rchit") || extension.eq("rmiss") || extension.eq("rgen") {
                let output = input.with_extension("spv");
                compile_shader(&input, &output);
            }
        }
    }
}

fn compile_shader(input: &PathBuf, output: &PathBuf) {
    let output = Command::new("glslc")
        .args(&[input.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .output()
        .expect("Failed to compile shader");

    if !output.status.success() {
        panic!("{}", std::str::from_utf8(&output.stderr).unwrap());
    }
}
