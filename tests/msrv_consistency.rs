//! Regression tests ensuring the declared Minimum Supported Rust Version
//! (MSRV) stays consistent across every file that documents or enforces it.
//!
//! This PR bumped the toolchain from `1.88.0` to `1.97.1` in six different
//! places (`Cargo.toml`, `rust-toolchain.toml`, `clippy.toml`, `README.md`,
//! `CONTRIBUTING.md`, and `.github/workflows/rust.yml`). These tests guard
//! against future bumps that update some files but miss others.

use std::fs;
use std::path::{Path, PathBuf};

/// The MSRV that all project files are expected to agree on.
const EXPECTED_MSRV: &str = "1.97.1";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative_path: &str) -> String {
    let path: PathBuf = manifest_dir().join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

/// Extracts the value of `key = "value"` (first match) from TOML-ish text.
fn extract_quoted_value(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim();
                if let Some(rest) = rest.strip_prefix('"') {
                    if let Some(end) = rest.find('"') {
                        return Some(rest[..end].to_string());
                    }
                }
            }
        }
    }
    None
}

#[test]
fn cargo_toml_declares_expected_rust_version() {
    let cargo_toml = read("Cargo.toml");
    let rust_version = extract_quoted_value(&cargo_toml, "rust-version")
        .expect("Cargo.toml must declare a `rust-version` key");
    assert_eq!(
        rust_version, EXPECTED_MSRV,
        "Cargo.toml `rust-version` should match the project MSRV"
    );
}

#[test]
fn rust_toolchain_toml_channel_matches_expected_msrv() {
    let toolchain_toml = read("rust-toolchain.toml");
    let channel = extract_quoted_value(&toolchain_toml, "channel")
        .expect("rust-toolchain.toml must declare a `channel` key");
    assert_eq!(
        channel, EXPECTED_MSRV,
        "rust-toolchain.toml `channel` should match the project MSRV"
    );
}

#[test]
fn rust_toolchain_toml_declares_required_components() {
    let toolchain_toml = read("rust-toolchain.toml");
    for component in ["rustfmt", "clippy", "rust-analyzer"] {
        assert!(
            toolchain_toml.contains(component),
            "rust-toolchain.toml should list the `{component}` component, got:\n{toolchain_toml}"
        );
    }
}

#[test]
fn clippy_toml_msrv_matches_expected_msrv() {
    let clippy_toml = read("clippy.toml");
    let msrv = extract_quoted_value(&clippy_toml, "msrv")
        .expect("clippy.toml must declare an `msrv` key");
    assert_eq!(
        msrv, EXPECTED_MSRV,
        "clippy.toml `msrv` should match the project MSRV"
    );
}

#[test]
fn readme_badge_and_prose_reference_expected_msrv() {
    let readme = read("README.md");
    assert!(
        readme.contains(&format!("rust-{EXPECTED_MSRV}-orange")),
        "README badge URL should reference rust-{EXPECTED_MSRV}-orange"
    );
    assert!(
        readme.contains(&format!("Rust {EXPECTED_MSRV}")),
        "README badge alt text should reference Rust {EXPECTED_MSRV}"
    );
    assert!(
        readme.contains(&format!("Rust **{EXPECTED_MSRV}**")),
        "README prose should reference Rust **{EXPECTED_MSRV}**"
    );
}

#[test]
fn contributing_md_references_expected_msrv() {
    let contributing = read("CONTRIBUTING.md");
    assert!(
        contributing.contains(&format!("**{EXPECTED_MSRV}**")),
        "CONTRIBUTING.md should reference the current toolchain version **{EXPECTED_MSRV}**"
    );
}

#[test]
fn workflow_pins_expected_toolchain_in_every_job() {
    let workflow = read(".github/workflows/rust.yml");
    let expected_line = format!("toolchain: {EXPECTED_MSRV}");
    let occurrences = workflow.matches(expected_line.as_str()).count();

    // format, clippy, check, unit-test, and doc jobs each pin a toolchain.
    assert_eq!(
        occurrences, 5,
        "expected exactly 5 jobs pinning `{expected_line}` in rust.yml, found {occurrences}"
    );

    // Every `toolchain:` line in the workflow must use the same version;
    // otherwise CI jobs would silently run against mismatched toolchains.
    for line in workflow.lines().filter(|l| l.trim_start().starts_with("toolchain:")) {
        assert!(
            line.contains(EXPECTED_MSRV),
            "found a toolchain pin that does not match the expected MSRV: {line}"
        );
    }
}

#[test]
fn no_stale_references_to_previous_toolchain_version() {
    let stale_version = "1.88";
    for relative_path in [
        "Cargo.toml",
        "rust-toolchain.toml",
        "clippy.toml",
        "README.md",
        "CONTRIBUTING.md",
        ".github/workflows/rust.yml",
    ] {
        let contents = read(relative_path);
        assert!(
            !contents.contains(stale_version),
            "{relative_path} still references the stale toolchain version {stale_version}"
        );
    }
}

#[test]
fn all_msrv_declaring_files_agree_with_each_other() {
    let cargo_toml = read("Cargo.toml");
    let toolchain_toml = read("rust-toolchain.toml");
    let clippy_toml = read("clippy.toml");

    let cargo_version =
        extract_quoted_value(&cargo_toml, "rust-version").expect("Cargo.toml rust-version");
    let toolchain_version =
        extract_quoted_value(&toolchain_toml, "channel").expect("rust-toolchain.toml channel");
    let clippy_msrv = extract_quoted_value(&clippy_toml, "msrv").expect("clippy.toml msrv");

    assert_eq!(
        cargo_version, toolchain_version,
        "Cargo.toml rust-version and rust-toolchain.toml channel must match"
    );
    assert_eq!(
        cargo_version, clippy_msrv,
        "Cargo.toml rust-version and clippy.toml msrv must match"
    );
}

#[test]
fn extract_quoted_value_returns_none_for_missing_key() {
    let text = "channel = \"1.97.1\"\ncomponents = [\"rustfmt\"]\n";
    assert_eq!(extract_quoted_value(text, "does-not-exist"), None);
}

#[test]
fn extract_quoted_value_ignores_key_substrings() {
    // A key like `msrv` should not accidentally match `some-msrv-like-key`.
    let text = "some-msrv-like-key = \"9.9.9\"\nmsrv = \"1.97.1\"\n";
    assert_eq!(
        extract_quoted_value(text, "msrv"),
        Some("1.97.1".to_string())
    );
}

#[test]
fn changed_files_exist_at_expected_paths() {
    for relative_path in [
        "Cargo.toml",
        "rust-toolchain.toml",
        "clippy.toml",
        "README.md",
        "CONTRIBUTING.md",
        ".github/workflows/rust.yml",
    ] {
        let path = manifest_dir().join(relative_path);
        assert!(
            Path::new(&path).is_file(),
            "expected {} to exist",
            path.display()
        );
    }
}