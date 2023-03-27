# Getting Started

In the following chapter, we will create a simple [postgresql] server using `tf-bindgen`, [Terraform] and [Docker]. Please make sure that all these tools are installed.

<!-- TODO: Transition -->

We will take a look at generation of Terraform bindings in section [1.1](./getting_started/generate). In section [1.2](./getting_started/resource.md) we will use these bindings to deploy our database. We will improve our source code using a helper macro in section [1.3](./getting_started/helpers.md) and at last we will create a construct for our database in section [1.4](./getting_started/construct.md).

In the following sections, we will create a new rust crate to run our deployment. The resulting source code can be found at [TODO](https://example.com).

[postgresql]: https://www.postgresql.org/
[Terraform]: https://www.terraform.io/
[Docker]: https://www.docker.com/

# Preparation

We will start with creating a new rust crate using the following command:

```sh
cargo new --bin "<name>" && cd "<name>"
```

This command will create a new directory at the current location using the specified project name.
In addition to the created project, we want to create another directory called `crates` in our directory. We will use this project to store our generated bindings and other libraries. To prepare this, we need to modify our `Cargo.toml` file.

```toml
# Add to your Cargo.toml file

[workspace]
members = []
```

The resulting project structure should look like:

```
<name>
 ├─ crates/
 │   └─ ...
 ├─ src/
 │   └─ main.rs
 └─ Cargo.toml
```

As last step, we will add `tf-bindgen` as dependency:

```sh
cargo add --git "https://github.com/robert-oleynik/tf-bindgen" "tf-bindgen"
```
