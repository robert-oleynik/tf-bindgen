use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=terraform.toml");

    let bindings = terraform_bindgen::Builder::default()
        .config("terraform.toml")
        .generate()
        .expect("failed to generate terraform bindings");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir, "terraform.rs").unwrap();
}
