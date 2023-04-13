# tf-bindgen

[![docs](https://img.shields.io/badge/docs-%20-success)](https://robert-oleynik.github.io/tf-bindgen/)
[![crates](https://img.shields.io/crates/v/tf-bindgen)](https://crates.io/crates/tf-bindgen)

<!-- Badges? -->

`tf-bindgen` can be used to generate Rust bindings for [Terraform] providers and
to deploy your infrastructure.
This library will replicate the features of [CDK for Terraform] but for Rust.

[Terraform]: https://www.terraform.io/
[CDK for Terraform]: https://developer.hashicorp.com/terraform/cdktf

## Requirements

Required tools:

- `cargo`
- `terraform`

## Usage

See [Documentation](https://robert-oleynik.github.io/tf-bindgen/introduction.html)

## Roadmap

**v0.1:**

<!-- Upcoming changes -->

- [x] generate Rust code for Terraform provider
  - [x] implement data blocks
  - [x] implement resource blocks
- [x] add support for variable references
- [x] generate Rust code from Terraform modules
- [x] add code generator `tf_bindgen::codegen::resource`
- [x] add Construct derive macro
- [x] create Markdown book

## Limitations

As mentioned above this library will replicate features provided by [CDK for Terraform].
It is not a one to one replacement for Rust and will be different in some aspects
of the implementation:

1. `cdktf` (library and CLI application) are not needed to use this library
2. `tf-bindgen` constructs are not compatible with CDK constructs.

## Contributing

<!-- TODO: add placeholder text -->

## License

This project is licensed under the [BSD-3-Clause](./LICENSE) license.
