use std::ops::{Deref, Index};

use serde::{Serialize, Serializer};

use super::prepare::Prepare;

/// Used to store field information and value.
#[derive(Clone)]
pub struct Cell<T> {
    path: String,
    value: T,
}

impl<T> Cell<T> {
    /// Creates a new cell using `name` as field name and value as content.
    pub fn new(name: impl Into<String>, value: impl Into<T>) -> Self {
        Self {
            path: name.into(),
            value: value.into(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> Index<usize> for Cell<Option<Vec<T>>> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.value {
            Some(arr) => &arr[index],
            None => panic!("cannot index empty array"),
        }
    }
}

impl<T: Prepare + Clone> Prepare for Cell<T> {
    fn prepare(self, prefix: impl Into<String>) -> Self {
        let path = prefix.into();
        let value = self.value.prepare(&path);
        Self { path, value }
    }
}

impl<T> Deref for Cell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Serialize> Serialize for Cell<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}
