# Getting Started

In this section, we will create a simple deployment of a [Postgres](https://www.postgresql.org/) database server. The resulting source code will be equivalent to the following Terraform HCL:

```terraform
trraform {
	requried_providers {
		docker = {
			source = "kreuzwerker/docker"
			version = "3.0.2"
		}
	}
}

provider "docker" {}

resource "docker_image" "postgres" {
	name = "postgres:latest"
}

resource "docker_container" "postgres" {
	image = docker_image.postgres.image_id
	name = "postgres"
	env = [
		"POSTGRES_PASSWORD=example"
	]
}
```

Before we start, we need to prepare our setup by adding the required dependencies:

```sh
cargo add tf-bindgen
cargo add tf-docker
```

For this section, we will not deal with generating our own bindings, but use some existing ones. If you are interested in this topic, you can read about it in [Generating Rust Bindings](./advanced/generation.md).

## Configure a Provider

We will start by setting up a stack to store our resources. Unlike in CDKTF, it is not necessary to create an `App`.

```rust
use tf_bindgen::Stack;

let stack = Stack::new("postgres");
```

After, we will configure our Docker provider. Because we use the default configuration, we do not have to call any setter and can create our provider immediately.

```rust
use tf_docker::Docker;

Docker::create(&stack).build();
```

Note that the provider will only be configured for the given stack.

## Create a Resource

After we configured our provider, we can use the resources and data sources provided by our bindings. In our case, we only use the `docker_image` and `docker_container` resource. We can import these to our deployment:

```rust
use tf_docker::resource::docker_image::*;
use tf_docker::resource::docker_container::*;
```

Equivalent to the code above: We can find correspond data sources under `tf_docker::data::<data source name>`.

We can use the imported configuration now, to create our docker image:

```rust
let image = DockerImage::create(&stack, "postgres-image")
	.name("postgres:latest")
	.build();
```

Using this snippet, we will create a new docker image to our stack. Important is that we have to specifiy an object id, in our case `"postgres-image"`, in addition to the associated stack. This ID is expected to be unique to the used scope, in this case the stack.

## Referencing an Attribute

In the next step, we will create our postgres container. To do that, we can use a similar builder exposed by `tf-docker`:

```rust
DockerContainer::create(&stack, "postgres-container")
	.name("postgres")
	.image(&image.image_id)
	.env(["POSTGRES_PASSWORD=example"])
	.build();
```

To use the Docker image ID generate by `"postgres-image"`, we have to reference the corresponding field in our image resource (similar to the HCL example).

If you play around with your Docker image configuration a bit, you may notice that you can not set the Image ID. This is because, `image_id` is a computed/read-only field exposed by Docker image resource (see [`docker_image` reference](https://registry.terraform.io/providers/kreuzwerker/docker/latest/docs/resources/image#read-only)).
