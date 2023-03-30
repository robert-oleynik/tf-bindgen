# Generate Rust Bindings

In this section, we will go over the things you need to do to set up custom generated bindings. We recommend using an empty library crate for doing so.

## Adding `tf-bindgen`

We will start by adding `tf-bindgen` to our Project. Because it is utilized by both our build script and the generated code, we need to add it twice. You can use the following command to add the latest version from the repository:

```sh
cargo add -p "docker" \
	--git "https://github.com/robert-oleynik/tf-bindgen.git" \
	"tf-bindgen"
cargo add --build -p "docker" \
	--git "https://github.com/robert-oleynik/tf-bindgen.git" \
	"tf-bindgen"
```

## Setup Build Script

As already mentioned, we will leverage Cargo's support for [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) to generate the bindings for our provider. Thereto, we will create a new `build.rs` in our crate:

```rust
// build.rs
use std::path::PathBuf;

fn main() {
	println!("cargo:rerun-if-changed=terraform.toml");

	let bindings = tf_bindgen::Builder::default()
		.config("terraform.toml")
		.generate()
		.unwrap();

	let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	bindings.write_to_file(out_dir, "terraform.rs").unwrap();
}
```

This script will read the provider specified in the `terraform.toml` file. In addition, it will parse the provider information and generate the corresponding Rust structs for it. The resulting bindings will be stored in the `terraform.rs` inside our build directory.

As you may have noticed, we did not create a `terraform.toml` yet. We will use this file to specify the providers we want to generate bindings for. A provider can be specified by adding `<provider name> = "<provider version>"` to the `[provider]` section of this TOML document. We will utilize the same version format as used by Cargo (see [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)). In the example below, we will use the [docker](https://registry.terraform.io/providers/kreuzwerker/docker/3.0.2) provider locked to version `3.0.2`:

```toml
# terraform.toml
[provider]
"kreuzwerker/docker" = "=3.0.2"
```

## Setup Module

Now we have generated our bindings, but we did not import them yet. To achieve that, we need to include the generated `terraform.rs` file into our crate.

```rust
// src/lib.rs
include!(concat!(env!("OUT_DIR"), "/terraform.rs"));
```

`tf-bindgen` will declare a module for each provider specified. So if you only declared a single provider, you may want to re-export these bindings to the current scope (e.g. `pub use docker::*;` in case of the docker provider).
