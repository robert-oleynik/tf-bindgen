use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;

use serde::{Serialize, Serializer};

#[derive(Clone)]
pub struct Cell<T> {
    path: String,
    value: Value<T>,
}

#[derive(Clone)]
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

impl<T> Deref for Cell<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value()
    }
}

macro_rules! value_from {
    (T: $t:ty) => {
        impl<T> From<$t> for Value<$t> {
            fn from(value: $t) -> Self {
                Self::Value(value)
            }
        }

        impl<T> From<Option<$t>> for Value<Option<$t>> {
            fn from(value: Option<$t>) -> Self {
                Self::Value(value)
            }
        }

        impl<T> From<$t> for Value<Option<$t>> {
            fn from(value: $t) -> Self {
                Self::Value(Some(value))
            }
        }
    };
    ($t:ty) => {
        impl From<$t> for Value<$t> {
            fn from(value: $t) -> Self {
                Self::Value(value)
            }
        }

        impl From<Option<$t>> for Value<Option<$t>> {
            fn from(value: Option<$t>) -> Self {
                Self::Value(value)
            }
        }

        impl From<$t> for Value<Option<$t>> {
            fn from(value: $t) -> Self {
                Self::Value(Some(value))
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
        Self::Value(value.to_string())
    }
}

impl<'a> From<&'a str> for Value<Option<String>> {
    fn from(value: &'a str) -> Self {
        Self::Value(Some(value.to_string()))
    }
}

impl<T> From<HashMap<String, T>> for Value<HashMap<String, T>> {
    fn from(value: HashMap<String, T>) -> Self {
        Self::Value(value)
    }
}

impl<T> From<Option<HashMap<String, T>>> for Value<Option<HashMap<String, T>>> {
    fn from(value: Option<HashMap<String, T>>) -> Self {
        Self::Value(value)
    }
}

impl<T: Clone> From<Option<Value<T>>> for Value<Option<T>> {
    fn from(value: Option<Value<T>>) -> Self {
        match value {
            Some(Value::Value(v)) => Self::Value(Some(v)),
            Some(Value::Ref(r)) => Self::Ref(Rc::new(Cell {
                path: r.path.clone(),
                value: Value::Value(Some(r.value().clone())),
            })),
            None => Self::Value(None),
        }
    }
}

impl<T: Clone> From<Option<Value<Option<T>>>> for Value<Option<T>> {
    fn from(value: Option<Value<Option<T>>>) -> Self {
        match value {
            Some(Value::Value(v)) => Self::Value(v),
            Some(Value::Ref(r)) => Self::Ref(Rc::new(Cell {
                path: r.path.clone(),
                value: Value::Value(r.value().clone()),
            })),
            None => Self::Value(None),
        }
    }
}

impl<T: Clone> From<&Rc<Cell<T>>> for Value<T> {
    fn from(value: &Rc<Cell<T>>) -> Self {
        Self::Ref(value.clone())
    }
}

pub fn serialize_rc_cell<S, T: Serialize>(
    this: &Rc<Cell<T>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    this.serialize(serializer)
}

impl<T: Serialize> Serialize for Cell<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.value {
            Value::Ref(r) => {
                let path = r.path();
                let id = format!("${{{path}}}");
                serializer.serialize_str(&id)
            }
            Value::Value(v) => v.serialize(serializer),
        }
    }
}
