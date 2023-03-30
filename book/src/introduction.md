# Introduction

> `tf-bindgen` is early in development. Expect breaking changes.

Before we start using `tf-bindgen`, we will explain a few concepts and ideas. We recommend basic knowledge about [Terraform](https://www.terraform.io/) and Infrastructure as Code (IaC). If you have no experience with Terraform, we also recommend reading up on the basic and some examples of Terraform HCL.
You can follow this [link](https://developer.hashicorp.com/terraform/intro) to start with an introduction provided by HashiCorp.

## What is `tf-bindgen`?

In 2022, HashiCorp released a tool called [CDK for Terraform](https://developer.hashicorp.com/terraform/cdktf) (CDKTF) to the public. It allows generating your IaC deployments using high-level languages, like Typescript, Python and Java, instead of relying on HashiCorp's declarative language HCL. That said, it does not come with support for Rust.

At this point, `tf-bindgen` comes into play. It will generate similar Rust bindings like CDKTF for Java, relying on heavy use of the builder pattern. On the downside, it does not use JSII like CDKTF, so it will _not_ integrate with CDKTF but coexists alongside it.

## How does `tf-bindgen` work?

Similar to CDKTF, `tf-bindgen` is using the provider schema provided by Terraform to generate bindings with a similar structure like HCL. Then these structures will be used to generate the Terraform configuration using JSON files. Terraform can use these files to plan and deploy your infrastructure. Simplified: `tf-bindgen` is like CDKTF, a glorified JSON generator for Terraform IaC.
