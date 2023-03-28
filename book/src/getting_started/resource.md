# Resources

We have prepared our `docker` bindings and can use them in our deployment now. But before we start, we have to set up our deployment environment. At first, we will create an `App` and a `Stack` to store our configuration.

```rust
use tf_bindgen::app::App;
use tf_bindgen::stack::Stack;

fn main() {
	let app = App::default();
	let stack = Stack::new(&app, "postgres");

	todo!();
}
```

## How Providers, Resources, and Data Sources are Generated

To use `tf-bindgen` it is beneficial to know how the structures of each binding is generated. In the following code block, you can see a resource declaration with different structures Terraform provides:

```terraform
resource "<resource_type>" "<resource_name>" {
	nested_block {
		argument = "value"
	}
	object = {
		arg1 = 0
		arg2 = false
	}
}
```

To replicate these structures in Rust, we will differentiate to type of structures:

- Nested structures like a `nested_block` or an `object` and
- simple structures like bool, string, numbers, and so on.

In general nested structures, resources, data sources and providers will use builders to set their internal fields. On the other hand, simple structures can be passed as an argument to field's setter function and will not require using a builder. So converting the resource declaration to rust looks like:

```rust
let nested_block = ResourceTypeNestedBlock::builder()
	.argument("value")
	.build();
let object = ResourceTypeObject::builder()
	.arg1(0)
	.arg2(false)
	.build();
ResourceType::create(&scope, "<resource_name>")
	.nested_block(nested_block)
	.object(object)
	.build();
```

You can see some naming conventions in the source code above:

- Nested types will be concatenated with their parent type name and their field name converted to camel case (e.g. parent type `ResourceType` and field name `nested_block` will be converted to `ResourceTypeNestedBlock`),
- each argument in Terraform will be a field in rust and
- in case an argument is named `build` it will be renamed to `build_`.

The same rules can be applied to providers and data sources.

## Set up the Docker Provider

Now there we know how the generator works, we will start with creating the default `docker` provider:

```rust
use docker::Docker;

fn main() {
	// ...
	Docker::create(&stack).build();
}
```

This code snippet will add the docker provider to our stack, so we can use docker resources for our deployment.

## Creating a Resource

Our goal for this section is to create the equivalent source code for the following HCL configuration:

```terraform
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

We will start with creating the Docker image:

```rust
use docker::resource::docker_image::*;

fn main() {
	// ...
	let image = DockerImage::create(&scope, "postgres-image")
		.name("postgres:latest")
		.build();
}
```

As you may have guessed, this code snippet will create the same `docker_image` resource as the HCL configuration above.

## Referencing Attributes

Now we can use the `docker_image` image resource to create our Docker container, by passing the image ID as reference to our container. In addition, we will set the name and the environment of our Docker container similar to our Docker image:

```rust
use docker::resource::docker_image::*;

fn main() {
	// ...
    DockerContainer::create(&stack, "postgres-container")
        .name("postgres")
        .image(&image.image_id)
        .env(["POSTGRES_PASSWORD=example"])
        .build();
}
```

Terraform will take care of the rest for us.

## Deploying our App

Our configuration is done for now. The only thing left to-do, is to deploy our app to docker. We can use the `app.deploy()` function for this:

```rust
fn main() {
	// ...
	app.deploy().unwrap();
}
```

Be aware that this function will only deploy resources created to this point, so it should be the last statement in main.
