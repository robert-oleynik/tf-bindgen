mod path;
mod stack;

use ::tf_schema::document::Resource;

pub use crate::path::Path;
pub use crate::stack::Stack;

pub trait Scope {
    /// Returns the stack an object is associated with.
    fn stack(&self) -> Stack;

    /// Returns the object path of `self`.
    fn path(&self) -> Path;
}

pub trait L1Construct: Scope {
    /// Returns the resource type and configuration of this construct.
    fn to_schema(&self) -> (String, Resource);
}

pub trait Provider: Scope {
    /// Returns the provider version and configuration.
    fn to_schema(&self) -> (String, tf_schema::document::Provider);
}
