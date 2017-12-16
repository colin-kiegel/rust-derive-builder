#![cfg(feature = "nightlytests")]
extern crate compiletest_rs as compiletest;

// note:
// - `env::var("PROFILE")` is only available vor build scripts
//   http://doc.crates.io/environment-variables.html
const PROFILE: &'static str = "debug";

use std::env;
use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let base_dir = env!("CARGO_MANIFEST_DIR");

    let test_dir = PathBuf::from(format!("{}/tests/{}", base_dir, mode));

    if !test_dir.is_dir() {
        panic!("Directory does not exist: {:?}", test_dir);
    }

    let mut config = compiletest::Config::default();
    let cfg_mode = mode.parse().ok().expect("Invalid mode");

    config.mode = cfg_mode;
    config.src_base = test_dir;

    // note:
    // - cargo respects the environment variable `env::var("CARGO_TARGET_DIR")`,
    //   however if this is not set and a virtual manifest is used, we will *not*
    //   know the path :-(
    // In that case try to set `CARGO_TARGET_DIR` manually, e.g.
    // `/path/to/my_workspace/target`.
    let build_dir = env::var("CARGO_TARGET_DIR").unwrap_or(format!("{}/target", base_dir));
    let artefacts_dir = format!("{}/{}", build_dir, PROFILE);

    config.target_rustcflags = Some(format!("-L {} -L {}/deps", artefacts_dir, artefacts_dir));

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("run-pass");
    run_mode("compile-fail");
}
