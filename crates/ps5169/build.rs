use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let generator = "../../profiles/generate_profile.py";
    println!("cargo:rerun-if-env-changed=PS5169_PROFILE");
    println!("cargo:rerun-if-changed={generator}");

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"))
        .join("ps5169_profile.rs");
    let Some(profile) = env::var_os("PS5169_PROFILE") else {
        fs::write(
            output,
            "pub const SELECTED_SEQUENCES: Option<RegisterSequences<&'static [u8]>> = None;\n",
        )
        .expect("write the default profile selection");
        return;
    };
    let profile = PathBuf::from(profile);
    println!("cargo:rerun-if-changed={}", profile.display());
    let status = Command::new("python3")
        .arg(generator)
        .arg(profile)
        .arg(output)
        .status()
        .expect("launch profile generator with Python 3.11 or later");
    assert!(status.success(), "PS5169 profile generation failed");
}
