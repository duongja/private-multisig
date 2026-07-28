use std::env;
use std::path::Path;

use spel_framework_core::idl_gen::generate_idl_from_file;

fn main() {
    let source = env::args()
        .nth(1)
        .expect("usage: spel_idl_exporter <wrapper.rs>");
    let idl =
        generate_idl_from_file(Path::new(&source)).expect("generate SPEL IDL from wrapper source");
    println!(
        "{}",
        serde_json::to_string_pretty(&idl).expect("serialize generated IDL")
    );
}
