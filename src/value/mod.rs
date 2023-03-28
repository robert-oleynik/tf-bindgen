use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

use serde::{Serialize, Serializer};

mod cell;
mod prepare;

pub use cell::Cell;
pub use prepare::Prepare;

#[derive(Clone)]
pub enum Value<T> {
    Ref { path: String, value: Box<Value<T>> },
    Value { value: Rc<T> },
    Computed,
}

impl<T> Value<T> {
    pub fn get(&self) -> Rc<T> {
        match &self {
            Value::Ref { value, .. } => value.get(),
            Value::Value { value } => value.clone(),
            Value::Computed => unimplemented!("computed values cannot be dereferenced"),
        }
    }
}

impl<T> Deref for Value<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Value::Ref { value, .. } => &value,
            Value::Value { value } => &value,
            Value::Computed => unimplemented!("computed values cannot be dereferenced"),
        }
    }
}

macro_rules! value_from {
    (T: $t:ty) => {
        impl<T> From<$t> for Value<$t> {
            fn from(value: $t) -> Self {
                Self::Value {
                    value: Rc::new(value),
                }
            }
        }

        impl<T> From<Option<$t>> for Value<Option<$t>> {
            fn from(value: Option<$t>) -> Self {
                Self::Value {
                    value: Rc::new(value),
                }
            }
        }

        impl<T> From<$t> for Value<Option<$t>> {
            fn from(value: $t) -> Self {
                Self::Value {
                    value: Rc::new(Some(value)),
                }
            }
        }
    };
    ($t:ty) => {
        impl From<$t> for Value<$t> {
            fn from(value: $t) -> Self {
                Self::Value {
                    value: Rc::new(value),
                }
            }
        }

        impl From<Option<$t>> for Value<Option<$t>> {
            fn from(value: Option<$t>) -> Self {
                Self::Value {
                    value: Rc::new(value),
                }
            }
        }

        impl From<$t> for Value<Option<$t>> {
            fn from(value: $t) -> Self {
                Self::Value {
                    value: Rc::new(Some(value)),
                }
            }
        }
    };
}

value_from!(bool);
value_from!(String);
value_from!(i64);
value_from!(serde_json::Value);
value_from!(T: HashSet<T>);
value_from!(T: Vec<T>);

impl<'a> From<&'a str> for Value<String> {
    fn from(value: &'a str) -> Self {
        Self::Value {
            value: Rc::new(value.to_string()),
        }
    }
}

impl<'a> From<&'a str> for Value<Option<String>> {
    fn from(value: &'a str) -> Self {
        Self::Value {
            value: Rc::new(Some(value.to_string())),
        }
    }
}

impl<T> From<T> for Value<Vec<T>> {
    fn from(value: T) -> Self {
        Self::Value {
            value: Rc::new(vec![value]),
        }
    }
}

impl<T> From<T> for Value<Option<Vec<T>>> {
    fn from(value: T) -> Self {
        Self::Value {
            value: Rc::new(Some(vec![value])),
        }
    }
}

impl<T> From<HashMap<String, T>> for Value<HashMap<String, T>> {
    fn from(value: HashMap<String, T>) -> Self {
        Self::Value {
            value: Rc::new(value),
        }
    }
}

impl<T> From<Option<HashMap<String, T>>> for Value<Option<HashMap<String, T>>> {
    fn from(value: Option<HashMap<String, T>>) -> Self {
        Self::Value {
            value: Rc::new(value),
        }
    }
}

impl<T: Hash + Eq, U: Into<T>, const L: usize> From<[U; L]> for Value<HashSet<T>> {
    fn from(values: [U; L]) -> Self {
        Self::Value {
            value: Rc::new(values.into_iter().map(Into::into).collect()),
        }
    }
}

impl<T: Hash + Eq, U: Into<T>, const L: usize> From<[U; L]> for Value<Option<HashSet<T>>> {
    fn from(values: [U; L]) -> Self {
        Self::Value {
            value: Rc::new(Some(values.into_iter().map(Into::into).collect())),
        }
    }
}

impl<T, const L: usize> From<[T; L]> for Value<Vec<T>> {
    fn from(values: [T; L]) -> Self {
        Self::Value {
            value: Rc::new(values.into_iter().map(Into::into).collect()),
        }
    }
}

impl<T, const L: usize> From<[T; L]> for Value<Option<Vec<T>>> {
    fn from(values: [T; L]) -> Self {
        Self::Value {
            value: Rc::new(Some(values.into_iter().map(Into::into).collect())),
        }
    }
}

impl<T: Clone> From<Option<Value<T>>> for Value<Option<T>> {
    fn from(value: Option<Value<T>>) -> Self {
        match value {
            Some(Value::Value { value }) => Self::Value {
                value: Rc::new(Some(value.deref().clone())),
            },
            Some(Value::Computed) => todo!(),
            Some(Value::Ref { path, value }) => Self::Ref {
                path: path.clone(),
                value: Box::new(Value::Value {
                    value: Rc::new(Some((**value).clone())),
                }),
            },
            None => Self::Value {
                value: Rc::new(None),
            },
        }
    }
}

impl<T: Clone> From<Option<Value<Option<T>>>> for Value<Option<T>> {
    fn from(value: Option<Value<Option<T>>>) -> Self {
        match value {
            Some(Value::Value { value }) => Self::Value {
                value: Rc::new(value.deref().clone()),
            },
            Some(Value::Ref { path, value }) => Self::Ref {
                path: path.clone(),
                value,
            },
            Some(Value::Computed) => todo!(),
            None => Self::Value {
                value: Rc::new(None),
            },
        }
    }
}

impl<T: Clone> From<&Cell<T>> for Value<T> {
    fn from(value: &Cell<T>) -> Self {
        Self::Ref {
            path: value.path().to_string(),
            value: Box::new(value.value().clone()),
        }
    }
}

impl<T: Serialize> Serialize for Cell<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.value() {
            Value::Ref { path, .. } => {
                let id = format!("${{{path}}}");
                serializer.serialize_str(&id)
            }
            Value::Value { value } => value.serialize(serializer),
            Value::Computed => unimplemented!("computed value can not been serialized"),
        }
    }
}
