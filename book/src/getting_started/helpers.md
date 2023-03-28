# Helpers

## Using the `resource!` Macro

While it is nice to have a builder to create resources, the syntax of HCL is still simple. That is why `tf_bindgen` comes with a `resource!` macro. It can be used to replicate the HCL syntax in rust. In the following snippet, we will replace the creation of our Docker container with this macro:

```rust
use docker::resource::docker_container::*;

fn main() {
	// ...
	tf_bindgen::codegen::resource! {
		&stack,
		resource "docker_container" "postgres-container" {
			name = "postgres"
			image = &image.image_id
			env = ["POSTGRES_PASSWORD=example"]
		}
	}
	// ...
}
```

**Important**: you still need to import all types associated with that import (see `docker::resource::docker_container::*`).
