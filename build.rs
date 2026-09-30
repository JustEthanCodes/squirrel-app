use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("squirrel_data.rs");

    let img_bytes = fs::read("squirrel.png").unwrap();
    let code = format!("pub const SQUIRREL_PNG: &[u8] = &{:?};", img_bytes);

    fs::write(&dest_path, code).unwrap();
    println!("cargo:rerun-if-changed=squirrel.png");
}
