extern crate compiletest_rs as compiletest;

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
    config.link_deps();
    config.clean_rmeta();

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("run-pass");
    run_mode("compile-fail");
}
