> You can skip this section, if you only plan to use pre-generated providers.

# Generate Bindings

Before we start with building our deployment, we need to select our bindings. In our case, we want to use docker bindings. For demonstration purposes, we will generate these bindings.

## Creating Bindings Crate

While it is possible to specify the dependencies inline, we recommend to use separate crates for them. So we will start with creating a new `docker` crate inside the `crates` directory:

```sh
cargo new --lib crates/docker
```

We still need to add the new crate to our project by adding it to the `Cargo.toml` file:

```toml
# ...
[workspace]
members = [
	"crates/docker"
]
```

In addition, we need to add the docker crate as dependency to our main binary using the following command:

```sh
cargo add --path "crates/docker"
```

To use `tf-bindgen` we have to add `tf-bindgen` as normal and build dependency to the docker crate:

```sh
cargo add -p "docker" \
	--git "https://github.com/robert-oleynik/tf-bindgen.git" \
	"tf-bindgen"
cargo add --build -p "docker" \
	--git "https://github.com/robert-oleynik/tf-bindgen.git" \
	"tf-bindgen"
```

## Setup Build Script

In this section, we will use Cargo's support for [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) to generate the bindings for the docker provider. Thereto, we will create a new Rust file with the following content at `crates/docker/build.rs` :

```rust
// crates/docker/build.rs
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

As you may have noticed, we did not create a `terraform.toml`. This file is used to configure the bindings generator and will store all provider with their version constraints. Providers can be specified by adding `<provider name> = "<provider version>"` to the `[provider]` section of the TOML document. It will utilize the same version format as used by Cargo (see [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)), We will use the [docker](https://registry.terraform.io/providers/kreuzwerker/docker/3.0.2) provider with version `3.0.2`, so our resulting `terraform.toml` looks like:

```toml
# crates/docker/terraform.toml
[provider]
"kreuzwerker/docker" = "=3.0.2"
```

## Setup Module

Now we can generate our bindings, but they are still not accessible from our main binary. To achieve that, we need to include the generated `terraform.rs` file into our `docker` crate.

```rust
// crates/docker/src/lib.rs
include!(concat!(env!("OUT_DIR"), "/terraform.rs"));

pub use docker::*;
```

In addition, we will reexport all declarations of the `terraform.rs` to simplify our module name.
