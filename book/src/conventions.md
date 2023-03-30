# Naming Conventions

## For Generated Structures

1. Type names of nested types will be created by concatenate parent type names to the nested type name and convert them to camel case (e.g. the type name of `container` inside `KubernetesPodSpec` will be `KubernetesPodSpecContainer`; see section [Complex Resources](./complex.md#create-a-complex-resource)).
2. Attributes will be named identically in HCL and `tf-bindgen` so you can stick to the documentation of Terraform (see [Terraform Registry](https://registry.terraform.io/)).
3. In case an Attribute is called `build` it will be renamed to `build_` to avoid conflicts with the `build` function of the builder.
