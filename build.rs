use std::env;
use std::process::Command;

fn main() {
    let mut build_version = env!("CARGO_PKG_VERSION").to_string();

    if let Some(date) = git_commit_date() {
        build_version.push('-');
        build_version.push_str(&date);
    }

    if let Some(hash) = git_commit_hash() {
        build_version.push('-');
        build_version.push_str(&hash);
    }

    let dirty = is_git_dirty();
    if dirty {
        build_version.push('+');

        println!("cargo:warning=version: {}", build_version);
    }

    println!("cargo:rustc-env=BUILD_VERSION={}", build_version);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/");
}

fn git_commit_date() -> Option<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%cd", "--date=format:%Y%m%d"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => {
            println!("cargo:warning=failed to get commit date");
            None
        }
    }
}

fn git_commit_hash() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => {
            println!("cargo:warning=failed to get commit hash");
            None
        }
    }
}

#[rustfmt::skip]
fn is_git_dirty() -> bool {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output();

    match output {
        Ok(output) if output.status.success() => !output.stdout.is_empty(),
        _ => false,
    }
}
