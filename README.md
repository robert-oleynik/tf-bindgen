# tf-bindgen

<!-- Badges? -->

`tf-bindgen` can be used to generate Rust bindings for [Terraform] providers and to deploy your infrastructure.
This library will replicate the features of [CDK for Terraform] but for Rust.

[Terraform]: https://www.terraform.io/
[CDK for Terraform]: https://developer.hashicorp.com/terraform/cdktf

## Requirements

Required tools:

-   `cargo`
-   `terraform`

## Usage

`tf-bindgen` can be used for two things:

1. deploying infrastructure and
2. generating bindings for Terraform provider

### Deploying Infrastructure

In the following section we will create a Kubernetes Pod running nginx. The deployment will be equivalent
to the following Terraform HCL code:

```hcl
terraform {
    required_providers = {
        kubernetes = {
            source = "hashicorp/kubernetes"
            version = "2.18.1"
        }
    }
}

resource "kubernetes_pod" "nginx" {
    metadata {
        name = "nginx"
    }
    spec {
        container {
            image = "nginx"
            name = "nginx"

            port {
                container_port = 80
            }
        }
    }
}
```

#### Using Provider Bindings

To deploy the nginx pod, we need to setup the Kubernetes provider. There are two ways to use Provider
bindings:

1. use pre-generated Bindings and
2. generate custom bindings (see [Generating Terraform Binding](#generating-terraform-bindings))

We will use pre-generated bindings for this example. You can add this bindings using cargo and running
the following command:

```sh
cargo add --git "https://github.com/robert-oleynik/tf-kubernetes.git"
```

We also need to add `tf-bindgen` to manage our deployment using the following command:

```sh
cargo add --git "https://github.com/robert-oleynik/tf-bindgen.git" "tf-bindgen"
```

#### Building Infrastructure

Using the provider bindings and `tf-bindgen` we can write our deployment now. We will start by creating
an app and a stack similar to [CDK for Terraform]:

```rust
use tf_bindgen::app::App;
use tf_bindgen::stack::Stack;

fn main() {
    let app = App::default();
    let stack = Stack::new(&app, "nginx");
}
```

We can create new Kubernetes Pod using a builder provided by the bindings:

```rust
// ...
use tf_kubernetes::kubernetes::resource::kubernetes_pod::{self, *};

fn main() {
    // ...
    let meta = kubernetes_pod::Metadata::builder().name("nginx").build();
    let port = kubernetes_pod::SpecContainerPort::builder()
        .container_port(80)
        .build();
    let container = kubernetes_pod::SpecContainer::builder()
        .name("nginx")
        .image("nginx")
        .port(vec![port])
        .build();
    let spec = kubernetes_pod::Spec::builder()
        .container(vec![container])
        .build();

    KubernetesPod::create(&stack, "nginx")
        .metadata(meta)
        .spec(spec)
        .build();
}
```

Now we only need to deploy our infrastructure. We can do this by adding `app.deploy()` to the end of
main:

```rust
// ...

fn main() {
    // ...
    app.deploy(false);
}
```

The resulting `main.rs`:

```rust
use tf_bindgen::app::App;
use tf_bindgen::stack::Stack;
use tf_kubernetes::kubernetes::resource::kubernetes_pod::{self, *};

fn main() {
    let app = App::default();
    let stack = Stack::new(&app, "nginx");

    let meta = kubernetes_pod::Metadata::builder().name("nginx").build();
    let port = kubernetes_pod::SpecContainerPort::builder()
        .container_port(80)
        .build();
    let container = kubernetes_pod::SpecContainer::builder()
        .name("nginx")
        .image("nginx")
        .port(vec![port])
        .build();
    let spec = kubernetes_pod::Spec::builder()
        .container(vec![container])
        .build();

    KubernetesPod::create(&stack, "nginx")
        .metadata(meta)
        .spec(spec)
        .build();

    app.deploy();
}
```

#### Deploying Infrastructure

To deploy the infrastructure run `cargo run`.

### Generating Terraform Bindings

To generate terraform bindings we will need to create a [build script] and specify the versions of the
providers we want to use.

[build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html

#### Prepare the Build Script

Before we can use `tf-bindgen` inside of our build script, we have to add the library as a build dependency:

```sh
cargo add --build --git "https://github.com/robert-oleynik/tf-bindgen.git" "tf-bindgen"
```

Now we can generate our bindings using `tf-bindgen`. Using the following code we will create a `terraform.rs`
file inside our build directory. This file is used to bundle the generated providers.

```rust
// build.rs
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=terraform.toml");

    let bindings = tf_bindgen::Builder::default()
        // File to read the providers from
        .config("terraform.toml")
        // Generate the rust bindings
        .generate()
        .expect();

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir, "terraform.rs");
}
```

You can add the generated providers to your project by including the generated file:

```rust
include!(concat!(env!("OUT_DIR"), "/terraform.rs"));
```

#### Selecting Providers

As mentioned above we will use a `terraform.toml` file to select our providers. See the following lines
for an example configuration:

```toml
[provider]
kubernetes = "2.18.1"
```

The version of a provider must be specified in the same format used by Cargo.

#### Building Providers

The configured providers can be build by running `cargo build`.

## Roadmap

**v0.1:**

<!-- Upcoming changes -->

-   [x] generate Rust code for Terraform provider
    -   [x] implement data blocks
    -   [x] implement resource blocks
-   [x] implement Terraform cli wrapper as part of `App`
-   [ ] add support for variable references
-   [ ] add support for outputs
-   [ ] generate Rust code from Terraform modules

## Limitations

As mentioned above this library will replicate features provided by [CDK for Terraform]. It is not a
one to one replacement for Rust and will be different in some aspects of the implementation:

1. `cdktf` (library and CLI application) are not needed to use this library
2. `tf-bindgen` constructs are not compatible with CDK constructs.

## Contributing

<!-- TODO: add placeholder text -->

## License

This project is licensed under the [BSD-3-Clause](./LICENSE) license.
