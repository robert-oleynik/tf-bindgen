# Structure of `tf-bindgen`

`tf-bindgen` is a collection of multiple smaller crates building all the necessary features:

- `tf-core` Used to implement basic traits and structs required by the generated code and used by the other crates (e.g. `Stack` and `Scope`).
- `tf-codegen` Used to implement code generation tools to simplify usage of `tf-bindgen`.
- `tf-schema` The JSON schemas exposed by Terraform. Contains both [JSON Provider Schema](https://developer.hashicorp.com/terraform/cli/commands/providers/schema) and [JSON Configuration Schema](https://developer.hashicorp.com/terraform/language/syntax/json).
- `tf-cli` Used to implement Terraform CLI wrappers, which will take care of generating the JSON configuration and construction of the Terraform command.
- `tf-binding` Bundles the crates and implements the actual code generation.
