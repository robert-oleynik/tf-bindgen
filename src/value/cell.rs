use std::{ops::Deref, rc::Rc};

use super::{prepare::Prepare, Value};

/// Used to store field information and value.
#[derive(Clone)]
pub struct Cell<T> {
    path: String,
    value: Value<T>,
}

impl<T> Cell<T> {
    /// Creates a new cell using `name` as field name and value as content.
    pub fn new(name: impl Into<String>, value: impl Into<Value<T>>) -> Self {
        Self {
            path: name.into(),
            value: value.into(),
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn value(&self) -> &Value<T> {
        &self.value
    }

    pub fn get(&self) -> Rc<T> {
        self.value.get()
    }
}

impl<T: Prepare + Clone> Prepare for Cell<T> {
    fn prepare(self, prefix: impl Into<String>) -> Self {
        let path = prefix.into();
        let value = if let Value::Value { value } = self.value {
            let inner: T = value.deref().clone();
            let inner = inner.prepare(&path);
            Value::Value {
                value: Rc::new(inner),
            }
        } else {
            self.value
        };
        Self { path, value }
    }
}

impl<T> Deref for Cell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}
