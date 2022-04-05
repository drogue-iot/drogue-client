#[cfg(feature = "nom")]
mod parser;

#[cfg(feature = "nom")]
pub use parser::*;

use std::collections::HashMap;
#[cfg(feature = "nom")]
use std::convert::TryFrom;
use std::fmt;
use std::ops::Add;

#[derive(Default, Debug, PartialEq)]
pub struct LabelSelector(pub Vec<Operation>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Eq(String, String),
    NotEq(String, String),
    In(String, Vec<String>),
    NotIn(String, Vec<String>),
    Exists(String),
    NotExists(String),
}

#[cfg(feature = "nom")]
impl TryFrom<&str> for LabelSelector {
    type Error = parser::ParserError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(LabelSelector(parser::parse_from(value)?))
    }
}

#[cfg(feature = "nom")]
impl TryFrom<String> for LabelSelector {
    type Error = parser::ParserError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(LabelSelector(parser::parse_from(&value)?))
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (left, operation, right) = match self {
            Operation::Eq(l, r) => (l.clone(), "=".to_string(), r.clone()),
            Operation::NotEq(l, r) => (l.clone(), "!=".to_string(), r.clone()),
            Operation::In(l, r) => (
                l.clone(),
                " in ".to_string(),
                ["(", r.join(", ").as_str(), ")"].concat(),
            ),
            Operation::NotIn(l, r) => (
                l.clone(),
                " notin ".to_string(),
                ["(", r.join(", ").as_str(), ")"].concat(),
            ),
            Operation::Exists(l) => (l.clone(), "".to_string(), "".to_string()),
            Operation::NotExists(l) => ("!".to_string(), l.clone(), "".to_string()),
        };

        write!(f, "{}{}{}", left, operation, right)
    }
}

impl From<Operation> for LabelSelector {
    fn from(op: Operation) -> Self {
        LabelSelector(vec![op])
    }
}

impl std::ops::Add<Operation> for LabelSelector {
    type Output = LabelSelector;

    /// Add another operation to a label selector.
    ///
    fn add(mut self, op: Operation) -> Self {
        self.0.push(op);
        self
    }
}

// TODO is it possible to have a common implementation for
// hashmap<String, String> and Vec<(String, String)> ?
// maybe using the IntoIterator trait ?
impl<K, V> From<HashMap<K, V>> for LabelSelector
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    /// Convert a HashMap into a LabelSelctor with multiple operations.
    /// All the operations will be using the `Equals` operator.
    fn from(collection: HashMap<K, V>) -> Self {
        let mut selector = LabelSelector::new();

        for (key, value) in collection.into_iter() {
            selector = selector.add(Operation::Eq(
                key.as_ref().to_string(),
                value.as_ref().to_string(),
            ));
        }
        selector
    }
}

impl LabelSelector {
    pub fn new() -> Self {
        LabelSelector(Vec::new())
    }

    /// Convert a LabelSelector into query parameters for use with reqwest
    ///
    pub fn to_query_parameters(&self) -> Vec<(String, String)> {
        let mut labs = Vec::new();
        let _ = &self.0.iter().map(|op| labs.push(op.to_string()));

        let labs = labs.join(",");

        vec![("labels".to_string(), labs)]
    }
}

#[cfg(test)]
mod test {
    use crate::registry::v1::labels::{LabelSelector, Operation};
    use std::collections::HashMap;
    use std::ops::Add;

    #[test]
    fn test_serialize_equals_operation() {
        let op = Operation::Eq("zone".to_string(), "europe".to_string());
        assert_eq!(op.to_string(), "zone=europe");
    }

    #[test]
    fn test_serialize_not_equals_operation() {
        let op = Operation::NotEq("zone".to_string(), "europe".to_string());
        assert_eq!(op.to_string(), "zone!=europe");
    }

    #[test]
    fn test_serialize_in_operation() {
        let op = Operation::In(
            "country".to_string(),
            vec![
                "france".to_string(),
                "germany".to_string(),
                "serbia".to_string(),
            ],
        );
        assert_eq!(op.to_string(), "country in (france, germany, serbia)");
    }

    #[test]
    fn test_serialize_not_in_operation() {
        let op = Operation::NotIn(
            "country".to_string(),
            vec![
                "france".to_string(),
                "germany".to_string(),
                "serbia".to_string(),
            ],
        );
        assert_eq!(op.to_string(), "country notin (france, germany, serbia)");
    }

    #[test]
    fn test_serialize_exists_operation() {
        let op = Operation::Exists("power".to_string());
        assert_eq!(op.to_string(), "power");
    }

    #[test]
    fn test_serialize_not_exists_operation() {
        let op = Operation::NotExists("power".to_string());
        assert_eq!(op.to_string(), "!power");
    }

    #[test]
    fn test_from_map() {
        let selector = LabelSelector::new()
            .add(Operation::Eq("foo".to_string(), "bar".to_string()))
            .add(Operation::Eq("fizz".to_string(), "buzz".to_string()));

        let mut map = HashMap::new();
        map.insert("fizz", "buzz");
        map.insert("foo", "bar");
        let selector_from_map: LabelSelector = map.into();

        assert_eq!(selector_from_map, selector);
    }
}
