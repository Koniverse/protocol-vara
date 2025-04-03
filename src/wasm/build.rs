use invariant::InvariantProgram;
use sails_idl_gen::program;
use std::{env, fs::File, path::PathBuf};

fn main() {
    sails_rs::build_wasm();

    let manifest_dir_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let idl_file_path = manifest_dir_path.join("invariant.idl");

    let idl_file = File::create(idl_file_path).unwrap();

    program::generate_idl::<InvariantProgram>(idl_file).unwrap();
}
