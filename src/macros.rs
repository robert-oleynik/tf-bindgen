#[macro_export]
macro_rules! set {
    ($($value:expr),*$(,)?) => {
		{
			#[allow(dead_code)]
			use $crate::value::IntoValue;
			let mut set = ::std::collections::HashSet::<$crate::Value<_>>::new();
			$(
				let value = $value;
				set.insert(value.into_value());
			)*
			set
		}
	};
}

#[macro_export]
macro_rules! map {
    ($($key:literal = $value:expr),*$(,)?) => {
		{
			use $crate::value::IntoValue;
			let mut map = ::std::collections::HashMap::<::std::string::String, $crate::Value<_>>::new();
			$(
				let value = $value;
				map.insert($key.to_string(), value.into_value());
			)*
			map
		}
	};
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::Value;

    #[test]
    pub fn map_single() {
        let _map: HashMap<String, Value<String>> = crate::map! {
            "key" = "value"
        };
    }

    #[test]
    pub fn map_multi_value() {
        let _map: HashMap<String, Value<String>> = crate::map! {
            "key" = "value",
            "key2" = "value"
        };
    }

    #[test]
    pub fn map_trailing_comma() {
        let _map: HashMap<String, Value<String>> = crate::map! {
            "key" = "value",
        };
    }

    #[test]
    pub fn set_single() {
        let _map: HashSet<Value<String>> = crate::set! {
            "value"
        };
    }

    #[test]
    pub fn set_multi_value() {
        let _map: HashSet<Value<String>> = crate::set! {
            "value",
            "value2"
        };
    }

    #[test]
    pub fn set_trailing_comma() {
        let _map: HashSet<Value<String>> = crate::set! {
            "value",
        };
    }
}
