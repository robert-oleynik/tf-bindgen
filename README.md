# terraform-bindgen

<!-- Badges? -->

<!-- put description here -->

Generate rust code from [Terraform] providers and modules.

## Requirements

<!-- Required libraries and tools -->
- `cargo`
- `terraform`

## Usage

### Setup terraform-bindgen

`terraform-bindgen` is designed to be run as a build script (see [Cargo Reference](https://doc.rust-lang.org/cargo/reference/build-scripts.html)).
To use `terraform-bindgen` you need to add it as a build dependency. You can do this by running `cargo add --build terraform-bindgen`
or by modifying your `Cargo.toml`

```toml
# ...

[build-dependency]
terraform-bindgen = "0.1"
```

`terraform-bindgen` requires you to add a `build.rs` file to your project. The file will contain following
structure:

```rust
fn main() {
	println!("cargo:rerun-if-changed=terraform.toml");

	let bindings = terraform_bindgen::Builder::default()
		// Read terraform configuration from config file
		.config("terraform.toml")
		// Finish the builder and generate the bindings
		.generate()
		.expect("failed to generate terraform bindings");

	let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	bindings.write_to_file(out_dir, "terraform.rs").unwrap();
}
```

The generated bindings can be included by adding following code to your project:

```rust
include!(concat!(env!("OUT_DIR"), "/terraform.rs"));
```

## Roadmap

**v0.1:**

<!-- Upcoming changes -->
- [x] generate Rust code for Terraform provider
	- [x] implement data blocks
	- [x] implement resource blocks
- [x] implement Terraform cli wrapper as part of `App`
- [ ] add support for variable references
- [ ] add support for outputs
- [ ] generate Rust code from Terraform modules

## Contributing

<!-- TODO: add placeholder text -->

## License

This project is licensed under the [BSD-3-Clause](./LICENSE) license.
