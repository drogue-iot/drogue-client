use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A translator for the data sections of a resource.
///
/// The translator trait, in combination with the [`Dialect`] trait allows easy access to
/// spec and status section in a strongly typed fashion:
///
/// ```rust
/// use drogue_client::{dialect, Section, Translator};
/// use drogue_client::registry::v1::Application;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// pub struct FooSpec {
/// }
///
/// dialect!(FooSpec[Section::Spec => "foo"]);
///
/// fn work_with(app: &Application) {
///   match app.section::<FooSpec>() {
///     Some(Ok(foo)) => {
///         // foo section existed and could be parsed.
///     },
///     Some(Err(err)) => {
///         // foo section existed, but could not be parsed.
///     },
///     None => {
///         // foo section did not exist.
///     },
///   }
/// }
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
            Some(Err(err)) => return Err(err),
        };

        self.set_section(s)
    }

    fn clear_section<D>(&mut self)
    where
        D: Serialize + Dialect,
    {
        match D::section() {
            Section::Spec => self.spec_mut().remove(D::key()),
            Section::Status => self.status_mut().remove(D::key()),
        };
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
    /// The `spec` section.
    Spec,
    // The `status` section.
    Status,
}

/// A "dialect", of strongly typed main section.
pub trait Dialect {
    /// The name of the field inside the section.
    fn key() -> &'static str;
    /// The section.
    fn section() -> Section;
}

/// Implements the [`Dialect`] trait for a structure.
///
/// ```rust
/// use drogue_client::{dialect, Section};
///
/// pub struct FooSpec {
/// }
///
/// dialect!(FooSpec[Section::Spec => "foo"]);
/// ```
#[macro_export]
macro_rules! dialect {
    ($dialect:ty [ $section:expr => $key:literal ]) => {
        impl $crate::Dialect for $dialect {
            fn key() -> &'static str {
                $key
            }

            fn section() -> $crate::Section {
                $section
            }
        }
    };
}

/// A specific attribute of a dialected section.
pub trait Attribute {
    type Dialect: for<'de> Deserialize<'de> + Dialect;
    type Output;

    fn extract(dialect: Option<Result<Self::Dialect, serde_json::Error>>) -> Self::Output;
}

/// Implements the [`Attribute`] trait for an attribute/field of a dialect.
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

/// Implements the [`Translator`] trait for a structure.
///
/// This macro requires that the struct has a `spec` and `status` field of type `Map<String, Value>`.
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
