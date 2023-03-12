use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct Cell<T> {
    path: String,
    value: Value<T>,
}

pub enum Value<T> {
    Ref(Rc<Cell<T>>),
    Value(T),
}

impl<T> Cell<T> {
    pub fn new(path: impl Into<String>, value: impl Into<Value<T>>) -> Rc<Self> {
        Rc::new(Self {
            path: path.into(),
            value: value.into(),
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn value(&self) -> &T {
        self.value.get()
    }
}

impl<T> Value<T> {
    pub fn get(&self) -> &T {
        match &self {
            Value::Ref(r) => r.value(),
            Value::Value(v) => &v,
        }
    }
}

impl From<bool> for Value<bool> {
    fn from(value: bool) -> Self {
        Self::Value(value)
    }
}

impl From<String> for Value<String> {
    fn from(value: String) -> Self {
        Self::Value(value)
    }
}

impl From<serde_json::Value> for Value<serde_json::Value> {
    fn from(value: serde_json::Value) -> Self {
        Self::Value(value)
    }
}

impl<T> From<HashSet<T>> for Value<HashSet<T>> {
    fn from(value: HashSet<T>) -> Self {
        Self::Value(value)
    }
}

impl<T> From<HashMap<String, T>> for Value<HashMap<String, T>> {
    fn from(value: HashMap<String, T>) -> Self {
        Self::Value(value)
    }
}

impl<T> From<Vec<T>> for Value<Vec<T>> {
    fn from(value: Vec<T>) -> Self {
        Self::Value(value)
    }
}

impl<T> From<Option<T>> for Value<Option<T>> {
    fn from(value: Option<T>) -> Self {
        Self::Value(value)
    }
}

impl<T> From<Rc<Cell<T>>> for Value<T> {
    fn from(value: Rc<Cell<T>>) -> Self {
        Self::Ref(value)
    }
}
