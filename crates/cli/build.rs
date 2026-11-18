use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../../macos/cursor/main.m");
    println!("cargo:rerun-if-changed=../../macos/cursor/Info.plist");
    println!("cargo:rerun-if-changed=../../assets/terminal-hosts.json");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        other => panic!("Unsupported cursor tracker architecture: {other}"),
    };
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("figfig-cursor");
    let status = Command::new("xcrun")
        .arg("clang")
        .arg(format!(
            "-DFIGFIG_HOSTS_JSON={:?}",
            std::fs::read_to_string("../../assets/terminal-hosts.json").unwrap()
        ))
        .args([
            "-fobjc-arc",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Wno-unused-parameter",
            "-mmacosx-version-min=13.0",
            "-arch",
            arch,
            "-framework",
            "AppKit",
            "-framework",
            "Carbon",
            "-framework",
            "InputMethodKit",
            "-framework",
            "ApplicationServices",
            "../../macos/cursor/main.m",
            "-o",
        ])
        .arg(output)
        .status()
        .expect("Build cursor tracker with Xcode Command Line Tools");
    assert!(
        status.success(),
        "Could not compile the macOS cursor tracker"
    );
}
