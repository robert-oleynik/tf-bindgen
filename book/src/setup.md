# Setup

## Required Tools

Before we start, we need the following tools installed:

- [terraform](https://www.terraform.io/): used to run our deployment
- [cargo](https://doc.rust-lang.org/cargo/): used to run and manage our rust project

## Add `tf-bindgen`

It is recommended to add `tf-bindgen` as dependency to the `Cargo.toml` file, by running the following command:

```sh
cargo add "tf-bindgen"
```

Because `tf-bindgen` is still early in development, you may want to use the latest version from the main branch. You can add this by running the following command instead:

```sh
cargo add --git "https://github.com/robert-oleynik/tf-bindgen" "tf-bindgen"
```

While it is now possible to use `tf-bindgen` in your project, we still need the required Terraform provider bindings to build the deployment. There are two types of provider bindings:

- self generated bindings by `tf-bindgen`
- pre-generated bindings

We will discuss self generated bindings in a later chapter [1.1](./getting_started/generate.md). To use pre-generated bindings instead, you can add these as a dependency to your project, like `tf-bindgen`. You can find a list with official and community bindings in [chapter Pre-Generated Bindings](bindings.md).
