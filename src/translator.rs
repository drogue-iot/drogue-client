use serde::Deserialize;
use serde_json::{Map, Value};

pub trait Translator {
    fn spec(&self) -> &Map<String, Value>;
    fn status(&self) -> &Map<String, Value>;

    fn section<D>(&self) -> Option<Result<D, serde_json::Error>>
    where
        D: for<'de> Deserialize<'de> + Dialect,
    {
        match D::section() {
            Section::Spec => self.spec_for(D::key()),
            Section::Status => self.status_for(D::key()),
        }
    }

    fn spec_for<T, S>(&self, key: S) -> Option<Result<T, serde_json::Error>>
    where
        T: for<'de> Deserialize<'de>,
        S: AsRef<str>,
    {
        let result = self
            .spec()
            .get(key.as_ref())
            .map(|value| serde_json::from_value(value.clone()));

        result
    }

    fn status_for<T, S>(&self, key: S) -> Option<Result<T, serde_json::Error>>
    where
        T: for<'de> Deserialize<'de>,
        S: AsRef<str>,
    {
        let result = self
            .status()
            .get(key.as_ref())
            .map(|value| serde_json::from_value(value.clone()));

        result
    }

    fn attribute<A>(&self) -> A::Output
    where
        A: Attribute,
    {
        A::extract(self.section::<A::Dialect>())
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Section {
    Spec,
    Status,
}

pub trait Dialect {
    fn key() -> &'static str;
    fn section() -> Section;
}

pub trait Attribute {
    type Dialect: for<'de> Deserialize<'de> + Dialect;
    type Output;

    fn extract(dialect: Option<Result<Self::Dialect, serde_json::Error>>) -> Self::Output;
}

// attribute!(pub DeviceCore [ Name : bool ] => |v| match v {
//   Some(Ok(v)) => v,
//   Some(Err(_)) => false,
//   None => true,
// })
#[macro_export]
macro_rules! attribute {
    ($v:vis $dialect:ty [$name:ident : $output:ty] => | $value:ident | $($code:tt)* ) => {
        $v struct $name;

        impl $crate::Attribute for $name {
            type Dialect = $dialect;
            type Output = $output;

            fn extract(dialect: Option<Result<Self::Dialect, serde_json::Error>>) -> Self::Output {
                let $value = dialect;
                $($code)*
            }
        }
    };
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_json::Error;

    #[derive(Deserialize, Debug, Clone, Default)]
    pub struct Foo {
        pub spec: Map<String, Value>,
        pub status: Map<String, Value>,
    }

    impl Translator for Foo {
        fn spec(&self) -> &Map<String, Value> {
            &self.spec
        }

        fn status(&self) -> &Map<String, Value> {
            &self.status
        }
    }

    #[derive(Deserialize, Debug, Clone, Default)]
    pub struct Bar {
        pub name: String,
    }

    impl Dialect for Bar {
        fn key() -> &'static str {
            "bar"
        }

        fn section() -> Section {
            Section::Spec
        }
    }

    pub struct Name {}

    impl Attribute for Name {
        type Dialect = Bar;
        type Output = String;

        fn extract(dialect: Option<Result<Self::Dialect, Error>>) -> Self::Output {
            dialect
                .and_then(|d| d.map(|d| d.name).ok())
                .unwrap_or_default()
        }
    }

    #[test]
    fn test1() {
        let i = Foo::default();
        let _: Option<Result<Bar, _>> = i.section::<Bar>();
    }

    #[test]
    fn test_attr() {
        let i = Foo::default();
        let name: String = i.attribute::<Name>();
        assert_eq!(name, "");
    }
}
