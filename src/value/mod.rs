use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use serde::{Serialize, Serializer};

mod cell;
mod prelude;
mod prepare;

pub use cell::Cell;
pub use prelude::*;
pub use prepare::Prepare;

#[derive(Clone, PartialEq, Eq)]
pub enum Value<T> {
    Ref {
        path: String,
        value: Option<Box<Value<T>>>,
    },
    Value {
        value: Rc<T>,
    },
}

pub struct Computed<T> {
    _p: PhantomData<T>,
}

impl<T> Value<T> {
    pub fn get(&self) -> Rc<T> {
        match &self {
            Value::Ref {
                value: Some(value), ..
            } => value.get(),
            Value::Value { value } => value.clone(),
            _ => unimplemented!("can not unknown referenced values"),
        }
    }
}

impl<T> Deref for Value<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Value::Ref {
                value: Some(value), ..
            } => value,
            Value::Value { value } => value,
            _ => unimplemented!("can not dereference computed values"),
        }
    }
}

impl<T: Hash> Hash for Value<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Ref { path, .. } => {
                state.write(b"${");
                path.hash(state);
                state.write(b"}");
            }
            Value::Value { value } => value.hash(state),
        }
    }
}

impl<T> Default for Computed<T> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<T> Prepare for Computed<T> {
    fn prepare(self, _: impl Into<String>) -> Self {
        self
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self { _p: PhantomData }
    }
}

impl<T: Serialize> Serialize for Value<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Ref { path, .. } => format!("${{{path}}}").serialize(serializer),
            Value::Value { value } => value.serialize(serializer),
        }
    }
}

impl<T: Prepare + Clone> Prepare for Value<T> {
    fn prepare(self, prefix: impl Into<String>) -> Self {
        match self {
            Value::Ref { .. } => self,
            Value::Value { value } => Self::Value {
                value: value.prepare(prefix),
            },
        }
    }
}
