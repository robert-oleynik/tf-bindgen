use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;

use super::Cell;
pub use super::Value;

pub trait IntoValue<T> {
    fn into_value(self) -> Value<T>;
}

pub trait IntoValueList<T> {
    fn into_value_list(self) -> Vec<Value<T>>;
}

pub trait IntoValueSet<T> {
    fn into_value_set(self) -> HashSet<Value<T>>;
}

pub trait IntoValueMap<T> {
    fn into_value_map(self) -> HashMap<String, Value<T>>;
}

macro_rules! impl_into_value {
    ($t:ty) => {
        impl IntoValue<$t> for $t {
            fn into_value(self) -> Value<$t> {
                Value::Value {
                    value: Rc::new(self.into()),
                }
            }
        }
    };
}

impl_into_value!(bool);
impl_into_value!(i64);
impl_into_value!(String);
impl_into_value!(serde_json::value::Value);

impl IntoValue<String> for &str {
    fn into_value(self) -> Value<String> {
        self.to_string().into_value()
    }
}

impl<T> IntoValue<T> for Value<T> {
    fn into_value(self) -> Value<T> {
        self
    }
}

impl<T: Clone> IntoValue<T> for &Value<T> {
    fn into_value(self) -> Value<T> {
        self.clone()
    }
}

impl<T: Clone> IntoValue<T> for &Cell<Value<T>> {
    fn into_value(self) -> Value<T> {
        Value::Ref {
            path: self.path().to_string(),
            value: Some(Box::new(self.value().clone())),
        }
    }
}

impl<T: Clone> IntoValue<T> for &Cell<Option<Value<T>>> {
    fn into_value(self) -> Value<T> {
        Value::Ref {
            path: self.path().to_string(),
            value: self.value().as_ref().cloned().map(Box::new),
        }
    }
}

impl<T, U> IntoValueList<T> for Vec<U>
where
    U: IntoValue<T>,
{
    fn into_value_list(self) -> Vec<Value<T>> {
        self.into_iter().map(IntoValue::into_value).collect()
    }
}

impl<T, U, const S: usize> IntoValueList<T> for [U; S]
where
    U: IntoValue<T>,
{
    fn into_value_list(self) -> Vec<Value<T>> {
        self.into_iter().map(IntoValue::into_value).collect()
    }
}

impl<'a, T, U> IntoValueList<T> for &'a [U]
where
    U: IntoValue<T> + Clone,
{
    fn into_value_list(self) -> Vec<Value<T>> {
        self.iter().cloned().map(IntoValue::into_value).collect()
    }
}

impl<T, U> IntoValueSet<T> for HashSet<U>
where
    T: Hash + Eq,
    U: IntoValue<T>,
{
    fn into_value_set(self) -> HashSet<Value<T>> {
        self.into_iter().map(IntoValue::into_value).collect()
    }
}

impl<T, U, const S: usize> IntoValueSet<T> for [U; S]
where
    T: Hash + Eq,
    U: IntoValue<T>,
{
    fn into_value_set(self) -> HashSet<Value<T>> {
        self.into_iter().map(IntoValue::into_value).collect()
    }
}

impl<'a, T, U> IntoValueSet<T> for &'a [U]
where
    T: Hash + Eq,
    U: IntoValue<T> + Clone,
{
    fn into_value_set(self) -> HashSet<Value<T>> {
        self.iter().cloned().map(IntoValue::into_value).collect()
    }
}

impl<T, U> IntoValueMap<T> for HashMap<String, U>
where
    U: IntoValue<T>,
{
    fn into_value_map(self) -> HashMap<String, Value<T>> {
        self.into_iter()
            .map(|(key, value)| (key, value.into_value()))
            .collect()
    }
}

impl<T, U> IntoValueMap<T> for &HashMap<String, U>
where
    U: IntoValue<T> + Clone,
{
    fn into_value_map(self) -> HashMap<String, Value<T>> {
        self.iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .map(|(key, value)| (key, value.into_value()))
            .collect()
    }
}
