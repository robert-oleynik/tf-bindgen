# tf-bindgen

[![docs](https://img.shields.io/badge/docs-online-success)](https://robert-oleynik.github.io/tf-bindgen/)
[![crates](https://img.shields.io/crates/v/tf-bindgen)](https://crates.io/crates/tf-bindgen)

`tf-bindgen` can be used to generate Rust bindings for [Terraform] providers and
to deploy your infrastructure.
This library will replicate most features of [CDK for Terraform] but written in Rust.

[Terraform]: https://www.terraform.io/
[CDK for Terraform]: https://developer.hashicorp.com/terraform/cdktf

## Requirements

Required tools:

- `cargo`
- `terraform`

## What is `tf-bindgen`?

`tf-bindgen` is a code generator which will generate Rust code to configure infrastructure using [Terraform](https://www.terraform.io/). The following example shows how to use `tf-bindgen` to configure a Kubernetes pod running nginx:

```rust
fn init() -> Stack {
	let stack = Stack::new("nginx");

	/// Configure Resources using a builder
	let metadata = KubernetesNamespaceMetadata::builder()
		.name("nginx")
		.build();
	let namespace = KubernetesNamespace::create(&stack, "nginx-namespace")
		.metadata(metadata)
		.build();

	/// Configure Resources using the resource! macro
	resource! {
		&stack, resource "kubernetes_pod" "nginx" {
			metadata {
				namespace = &namespace.metadata[0].name
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
	};

	stack
}
```

See [Documentation](https://robert-oleynik.github.io/tf-bindgen/introduction.html) for a full introduction into `tf-bindgen`.

## Issues

### Compile Performance

Some Providers like [tf-kubernetes](https://github.com/robert-oleynik/tf-kubernetes) will generate large bindings, resulting in long compile durations. If you have this issue, please see the [Improving Compile Duration](https://robert-oleynik.github.io/tf-bindgen/advanced/improving_compile_duration.html) section.

## Roadmap

**v0.1:**

- [x] generate Rust code for Terraform provider
  - [x] implement data blocks
  - [x] implement resource blocks
- [x] add support for variable references
- [x] generate Rust code from Terraform modules
- [x] add code generator `tf_bindgen::codegen::resource`
- [x] add Construct derive macro
- [x] create Markdown book

**v0.2:**

- [ ] Add support for Outputs in constructs
- [ ] Add Macro to generate CLI application
- [ ] Add `format!` for Value types
- [ ] Remove derive macros from generated source code

## Limitations

As mentioned above, this library will replicate features provided by [CDK for Terraform].
It is not a one to one replacement for Rust and will be different in some aspects
of the implementation:

1. `cdktf` (library and CLI application) are not needed to use this library
2. `tf-bindgen` constructs are not compatible with CDK constructs.

## Contributing

<!-- TODO: add placeholder text -->

## License

This project is licensed under the [BSD-3-Clause](./LICENSE) license.
