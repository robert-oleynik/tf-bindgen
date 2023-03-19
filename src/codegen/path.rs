use heck::ToUpperCamelCase;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct Path {
    segments: Vec<String>,
}

impl Path {
    pub fn empty() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Concatenate path segments as camel case to type.
    pub fn type_name(&self) -> String {
        self.segments
            .iter()
            .map(|seg| seg.to_upper_camel_case())
            .collect()
    }

    /// Concatenate path segments as camel case to type. Will ignore the first segments.
    pub fn type_name_reduced(&self) -> String {
        self.segments
            .iter()
            .skip(1)
            .map(|seg| seg.to_upper_camel_case())
            .collect()
    }

    /// Concatenate path segments with dot separator.
    pub fn path_ref(&self) -> String {
        self.segments.iter().join(".")
    }

    pub fn push(&mut self, path: impl Into<String>) {
        self.segments.push(path.into())
    }
}
