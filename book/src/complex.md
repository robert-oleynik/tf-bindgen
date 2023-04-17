# Complex Resources

While creating a Docker image and container is straight forward, it may become a bit harder than using heavily nested resources (e.g. Kubernetes resources). In this section, we will create a Kubernetes pod with a NGINX container running inside. It will be equivalent to the following HCL code:

```terraform
terraform {
	required_providers = {
		kubernetes = {
			source = "kubernetes"
			version = "3.0.2"
		}
	}
}

provider "kubernetes" {
	config_path = "~/.kube/config"
}

resource "kubernetes_pod" "nginx" {
	metadata {
		name = "nginx"
	}
	spec {
		container {
			name = "nginx"
			image = "nginx"
			port {
				container_port = 80
			}
		}
	}
}
```

Similar to the docker example in the previous section, we need to add our dependencies:

```sh
cargo add tf-bindgen
cargo add tf-kubernetes
```

In this case, we will use the Kubernetes bindings instead of the Docker bindings. We also will start similar to the docker example by setting up our stack and Kubernetes provider:

```rust
use tf_bindgen::Stack;
use tf_kubernetes::Kubernetes;
use tf_kubernetes::resource::kubernetes_pod::*;

let stack = Stack::new("postgres");

Kubernetes::create(&stack)
	.config_path("~/.kube/config")
	.build();
```

## Create a Complex Resource

You have already seen how to set attributes of Terraform resources. We will use the same setters, like `config_path`, to set our nested structures. Important is that we have to create these nested structures, before we can pass them to the setter.

```rust
let metadata = KubernetesPodMetadata::builder()
	.name("nginx")
	.build();
```

As this snippet shows, the exposed builder will not need a scope or an ID to create a nested type. In addition, we will not use `create` but rather `builder` to create our builder object.

We can repeat this for our nested type `spec` :

```rust
let port = KubernetesPodSpecContainerPort::builder()
	.container_port(80)
	.build();
let container = KubernetesPodSpecContainer::builder()
	.name("nginx")
	.image("nginx")
	.port(port)
	.build();
let spec = KubernetesPodSpec::builder()
	.container(vec![container])
	.build();
```

You can notice a few things:

- Nested types of nested types will be created using the same builder pattern,
- they can be passed using the setter, and
- we can pass multiple nested types by using an array (in our case using `vec!`).

To finalize it, we can use our created `metadata` and `spec` object and pass it to our Kubernetes pod resource:

```rust
KubernetesPod::create(&stack, "nginx")
	.metadata(metadata)
	.spec(spec)
	.build();
```

## Using `tf_bindgen::codegen::resource`

While using builder is a nice way to set these attributes and allows very flexible code, using multiple builders in a row result in higher complexity than using HCL. That is the reason we implemented `tf_bindgen::codegen::resource` macro. It allows using a simplified HCL syntax to create resources. Our Kubernetes pod resource using this macro would look like:

```rust
tf_bindgen::codegen::resource! {
	&stack,
	resource "kubernetes_pod" "nginx" {
		metadata {
			name = "nginx"
		}
		spec {
			container {
				name = "nginx"
				image = "nginx"
				port {
					container_port = 80
				}
			}
		}
	}
}
```

This macro use the same builders as shown in the section before and will return the resulting resource.
