use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A translator for the data sections of a resource.
pub trait Translator {
    fn spec(&self) -> &Map<String, Value>;
    fn status(&self) -> &Map<String, Value>;

    fn spec_mut(&mut self) -> &mut Map<String, Value>;
    fn status_mut(&mut self) -> &mut Map<String, Value>;

    fn section<D>(&self) -> Option<Result<D, serde_json::Error>>
    where
        D: for<'de> Deserialize<'de> + Dialect,
    {
        match D::section() {
            Section::Spec => self.spec_for(D::key()),
            Section::Status => self.status_for(D::key()),
        }
    }

    fn set_section<D>(&mut self, d: D) -> Result<(), serde_json::Error>
    where
        D: Serialize + Dialect,
    {
        let v = serde_json::to_value(d)?;

        match D::section() {
            Section::Spec => self.spec_mut().insert(D::key().to_string(), v),
            Section::Status => self.status_mut().insert(D::key().to_string(), v),
        };

        Ok(())
    }

    fn update_section<D, F>(&mut self, f: F) -> Result<(), serde_json::Error>
    where
        D: Serialize + for<'de> Deserialize<'de> + Dialect + Default,
        F: FnOnce(D) -> D,
    {
        let s = match self.section::<D>() {
            Some(Ok(s)) => f(s),
            None => {
                let s = D::default();
                f(s)
            }
            Some(Err(err)) => Err(err)?,
        };

        self.set_section(s)
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

/// An enum of main data sections.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Section {
    Spec,
    Status,
}

/// A "dialect", of strongly types main sections.
pub trait Dialect {
    fn key() -> &'static str;
    fn section() -> Section;
}

/// A specific attribute of a dialected section.
pub trait Attribute {
    type Dialect: for<'de> Deserialize<'de> + Dialect;
    type Output;

    fn extract(dialect: Option<Result<Self::Dialect, serde_json::Error>>) -> Self::Output;
}

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

#[macro_export]
macro_rules! translator {
    ($name:ty) => {
        impl Translator for $name {
            fn spec(&self) -> &Map<String, Value> {
                &self.spec
            }

            fn status(&self) -> &Map<String, Value> {
                &self.status
            }

            fn spec_mut(&mut self) -> &mut Map<String, Value> {
                &mut self.spec
            }

            fn status_mut(&mut self) -> &mut Map<String, Value> {
                &mut self.status
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

    translator!(Foo);

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
