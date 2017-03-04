#![cfg(dev_nightly)]
extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let base_dir = env!("CARGO_MANIFEST_DIR");
    let test_dir = PathBuf::from(format!("{}/tests/{}", base_dir, mode));

    if test_dir.is_dir() {
        let mut config = compiletest::default_config();
        let cfg_mode = mode.parse().ok().expect("Invalid mode");

        config.mode = cfg_mode;
        config.src_base = test_dir;

        let profile = "debug";
        // note:
        // - `env::var("PROFILE")` is only available vor build scripts
        //   http://doc.crates.io/environment-variables.html

        config.target_rustcflags =
            Some(format!("-L target/{} -L target/{}/deps", profile, profile));

        compiletest::run_tests(&config);
    }
}

#[test]
fn compile_test() {
    run_mode("run-pass");
    run_mode("run-fail");
    run_mode("compile-fail");
}
