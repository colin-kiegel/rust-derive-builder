extern crate skeptic;

fn main() {
    println!("INFO: Run with `RUST_LOG=build_script_build=trace` for debug information.");
    skeptic::generate_doc_tests(&["README.md"]);
}
