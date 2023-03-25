use std::collections::{HashMap, HashSet};

pub trait Prepare {
    fn prepare(self, prefix: impl Into<String>) -> Self;
}

macro_rules! noop {
    ($t:ty) => {
        impl Prepare for $t {
            fn prepare(self, _: impl Into<String>) -> Self {
                self
            }
        }
    };
}

noop!(String);
noop!(bool);
noop!(i64);
noop!(serde_json::Value);

impl<T: Prepare> Prepare for Option<T> {
    fn prepare(self, prefix: impl Into<String>) -> Self {
        self.map(|v| v.prepare(prefix))
    }
}

impl<T: Prepare> Prepare for Vec<T> {
    fn prepare(self, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        self.into_iter()
            .enumerate()
            .map(|(i, el)| {
                let path = format!("{prefix}.{i}");
                el.prepare(path)
            })
            .collect()
    }
}

impl<T> Prepare for HashSet<T> {
    fn prepare(self, _: impl Into<String>) -> Self {
        self
    }
}

impl<T> Prepare for HashMap<String, T> {
    fn prepare(self, _: impl Into<String>) -> Self {
        self
    }
}
