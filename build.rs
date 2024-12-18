use core::str;
use std::{path::PathBuf, process::Command};
use inline_colorization::*;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("pawl-js/build/index.html");
    match std::fs::exists(PathBuf::from(d)) {
        Ok(defined) => if defined {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("pawl-js/src/");
            println!("cargo::rerun-if-changed={}", d.to_string_lossy());

            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("pawl-js/package.json");
            println!("cargo::rerun-if-changed={}", d.to_string_lossy());
        } else {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("pawl-js/");
            // Use the `cc` crate to build a C file and statically link it.
            let output = Command::new("npm")
            .arg("ping")
            .output()
            .expect("{style_bold}{color_bright_red}ratchet-pawl build error:{color_reset}{style_reset} Node Package Manager not found, or npm registry unreachable! Ensure the system is configured with npm.");
        
            if !output.status.success() {
                print_warning(format!("Unable to locate npm, cannot complete build."));
                
                print_warning(format!("Failed with: {}", str::from_utf8(&output.stdout).unwrap()));
                panic!("NPM unavailable");
            } else {
                print_warning(format!("Located NPM for frontend build."));
            }
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("pawl-js/");
            // Use the `cc` crate to build a C file and statically link it.
            let output = Command::new("bash")
            .current_dir(d)
            .arg("-c")
            .arg(format!("npm install"))
            .output()
            .expect("Node Package Manager error! Check system logs.");
        
            if !output.status.success() {
                print_warning(format!("NPM build failed."));
                print_warning(format!("NPM build reported:{}", str::from_utf8(&output.stdout).unwrap()));
                panic!("NPM unavailable");
            } else {
                print_warning(format!("Installed **node_modules**"));
            }
        },
        Err(_) => panic!("Filesystem error."),
    }

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("pawl-js/");
    // Use the `cc` crate to build a C file and statically link it.
    let output = Command::new("npm")
        .arg("ping")
        .output()
        .expect("Node Package Manager not found! Ensure the system is configured with npm.");

    if !output.status.success() {
        print_warning(format!("Unable to locate npm, cannot complete build."));
        
        print_warning(format!("Failed with: {}", str::from_utf8(&output.stdout).unwrap()));
        panic!("NPM unavailable");
    } else {
        print_warning(format!("Located NPM for frontend build."));
    }

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("pawl-js/");
    // Use the `cc` crate to build a C file and statically link it.
    let output = Command::new("bash")
        .current_dir(d)
        .arg("-c")
        .arg(format!("npm run build"))
        .output()
        .expect("Node Package Manager error! Check system logs.");

    if !output.status.success() {
        print_warning(format!("NPM build failed."));
        print_warning(format!("NPM build reported:{}", str::from_utf8(&output.stdout).unwrap()));
        panic!("NPM unavailable");
    } else {
        print_warning(format!("Frontend build completed successfully!"));
    }
}

fn print_warning(s: String) {
    println!("cargo::warning={}", s);
}