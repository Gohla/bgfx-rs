extern crate glob;

use std::process::{Command};
use std::io::Write;
use std::fs;
use std::path::Path;

fn main() {
  println!("cargo:rerun-if-changed=build.rs");

  // Targets based on bgfx-sys/bgfx/scripts/shader.mk
  let vs_targets = [
    ShaderTarget::new("vertex", "dx9", "windows", Some("vs_3_0"), Some("3")),
    ShaderTarget::new("vertex", "dx11", "windows", Some("vs_4_0"), Some("3")),
    ShaderTarget::new("vertex", "essl", "nacl", None, None),
    ShaderTarget::new("vertex", "android", "android", None, None),
    ShaderTarget::new("vertex", "gl", "linux", Some("120"), None),
    ShaderTarget::new("vertex", "metal", "osx", Some("metal"), None),
    // PSSL compiler is not supported
    //ShaderTarget::new("vertex", "gles", "orbis", Some("pssl"), None),
    ShaderTarget::new("vertex", "vulkan", "linux", Some("spirv"), None),
  ];
  let fs_targets = [
    ShaderTarget::new("fragment", "dx9", "windows", Some("ps_3_0"), Some("3")),
    ShaderTarget::new("fragment", "dx11", "windows", Some("ps_4_0"), Some("3")),
    ShaderTarget::new("fragment", "nacl", "nacl", None, None),
    ShaderTarget::new("fragment", "android", "android", None, None),
    ShaderTarget::new("fragment", "gl", "linux", Some("120"), None),
    ShaderTarget::new("fragment", "metal", "osx", Some("metal"), None),
    // PSSL compiler is not supported
    //ShaderTarget::new("fragment", "gles", "orbis", Some("pssl"), None),
    ShaderTarget::new("fragment", "vulkan", "linux", Some("spirv"), None),
  ];

  println!("cargo:rerun-if-changed=examples");

  compile_example_shaders("vs", &vs_targets);
  compile_example_shaders("fs", &fs_targets);
}

fn compile_example_shaders(stype: &str, targets: &[ShaderTarget]) {
  let pattern = format!("examples/**/assets/*.{}.sc", stype);
  for entry in glob::glob(&pattern).unwrap() {
    let input_path = entry.unwrap();
    let assets_path = input_path.parent().unwrap();
    println!("cargo:rerun-if-changed={}", assets_path.to_str().unwrap());
    let example_dir = assets_path.parent().unwrap();

    for target in targets {
      let out_dir = format!("{}/out/{}", example_dir.to_str().unwrap(), target.name);
      fs::create_dir_all(&Path::new(&out_dir)).expect("Failed to create output directory");

      let input_filename = input_path.file_name().unwrap().to_str().unwrap();
      let out_file = format!("{}/{}.bin", out_dir, input_filename);
      compile_shader(input_path.to_str().unwrap(), &out_file, target);
    }
  }
}

fn compile_shader(input_file: &str, output_file: &str, target: &ShaderTarget) {
  let mut command = shaderc_command();
  command
    .arg("-f").arg(input_file)
    .arg("-o").arg(output_file)
    .arg("--type").arg(&target.stype)
    .arg("--platform").arg(&target.platform);

  if let &Some(ref profile) = &target.profile {
    command.arg("-p").arg(profile);
  }
  if let &Some(ref optimization) = &target.optimization {
    command.arg("-O").arg(optimization);
  }

  let output = command
    .output()
    .expect("Failed to compile shader");

  writeln!(std::io::stderr(), "{}", String::from_utf8_lossy(&output.stderr)).unwrap();
  assert!(output.status.success());

  println!("cargo:rerun-if-changed={}", input_file);
  println!("cargo:rerun-if-changed={}", output_file);
}

#[cfg(target_os = "windows")]
fn shaderc_command() -> Command {
  return Command::new("bgfx-sys/bgfx/tools/bin/windows/shaderc.exe")
}

#[cfg(target_os = "linux")]
fn shaderc_command() -> Command {
  return Command::new("bgfx-sys/bgfx/tools/bin/linux/shaderc")
}

#[cfg(target_os = "macos")]
fn shaderc_command() -> Command {
  return Command::new("bgfx-sys/bgfx/tools/bin/darwin/shaderc")
}

struct ShaderTarget {
  stype: String,
  name: String,
  platform: String,
  profile: Option<String>,
  optimization: Option<String>
}

impl ShaderTarget {
  fn new(
    stype: &str,
    name: &str,
    platform: &str,
    profile: Option<&str>,
    optimization: Option<&str>
  ) -> ShaderTarget {
    let profile = match profile {
      Some(str) => { Some(String::from(str)) }
      None => { None }
    };
    let optimization = match optimization {
      Some(str) => { Some(String::from(str)) }
      None => { None }
    };
    return ShaderTarget {
      stype: String::from(stype),
      name: String::from(name),
      platform: String::from(platform),
      profile: profile,
      optimization: optimization,
    };
  }
}