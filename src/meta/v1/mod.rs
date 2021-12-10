use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/*
 * This file might look like a some duplication of code. The main difference between
 * these two structs is that one has an "application" field, while the other doesn't.
 *
 * Rust doesn't make composing structs easy, so the simplest solution is to simply have two.
 *
 * Now both could be generated with a macro, to not repeat ourselves. However, macros need to
 * expand into a valid syntax tree element, which a list of fields is not. There is a macro pattern
 * called "muncher", which would allow use to create something like this. Then again, this isn't
 * really readable. Assuming that these structures are more often viewed than edited, it may be
 * simpler to keep them as they are.
 *
 * Should the need for processing both scoped and non-scoped metadata using the same method, we
 * would need to implement a `Metadata` and `MetadataMut` trait, which provides a common way to
 * access the metadata. For now, we don't require this.
 */

fn epoch() -> DateTime<Utc> {
    Utc.timestamp_millis(0)
}

/// Non-scoped metadata.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct NonScopedMetadata {
    /// The unique name of this resource.
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub uid: String,

    /// The creation date of the resource.
    #[serde(default = "epoch")]
    pub creation_timestamp: DateTime<Utc>,
    /// A generation number, incrementing with each change.
    ///
    /// This increment between different version may be one, or greater than one.
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub resource_version: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion_timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub finalizers: Vec<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub annotations: HashMap<String, String>,
}

/// Application-scoped metadata.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ScopedMetadata {
    /// The application this resource is scoped by.
    pub application: String,
    /// The unique name of this resource.
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub uid: String,

    #[serde(default = "epoch")]
    pub creation_timestamp: DateTime<Utc>,
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub resource_version: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletion_timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub finalizers: Vec<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub annotations: HashMap<String, String>,
}

impl Default for ScopedMetadata {
    fn default() -> Self {
        Self {
            application: Default::default(),
            name: Default::default(),
            uid: Default::default(),
            labels: Default::default(),
            annotations: Default::default(),
            creation_timestamp: chrono::Utc::now(),
            resource_version: Default::default(),
            generation: Default::default(),
            deletion_timestamp: Default::default(),
            finalizers: Default::default(),
        }
    }
}

impl Default for NonScopedMetadata {
    fn default() -> Self {
        Self {
            name: Default::default(),
            uid: Default::default(),
            labels: Default::default(),
            annotations: Default::default(),
            creation_timestamp: chrono::Utc::now(),
            resource_version: Default::default(),
            generation: Default::default(),
            deletion_timestamp: Default::default(),
            finalizers: Default::default(),
        }
    }
}

/// A trait for immutable access to the common parts of the metadata structures.
pub trait CommonMetadata {
    fn name(&self) -> &String;
    fn uid(&self) -> &String;
    fn labels(&self) -> &HashMap<String, String>;
    fn annotations(&self) -> &HashMap<String, String>;
    fn creation_timestamp(&self) -> &DateTime<chrono::Utc>;
    fn resource_version(&self) -> &String;
    fn generation(&self) -> u64;
    fn deletion_timestamp(&self) -> &Option<DateTime<chrono::Utc>>;
    fn finalizers(&self) -> &Vec<String>;
}

/// A trait for mutable access to the common parts of the metadata structures.
pub trait CommonMetadataMut: CommonMetadata {
    fn set_name(&mut self, name: String);
    fn set_uid(&mut self, uid: String);
    fn set_labels(&mut self, labels: HashMap<String, String>);
    fn set_annotations(&mut self, annotations: HashMap<String, String>);
    fn set_creation_timestamp(&mut self, creation_timestamp: DateTime<Utc>);
    fn set_resource_version(&mut self, resource_version: String);
    fn set_generation(&mut self, generation: u64);
    fn set_deletion_timestamp(&mut self, deletion_timestamp: Option<DateTime<Utc>>);
    fn set_finalizers(&mut self, finalizers: Vec<String>);

    fn name_mut(&mut self) -> &mut String;
    fn uid_mut(&mut self) -> &mut String;
    fn labels_mut(&mut self) -> &mut HashMap<String, String>;
    fn annotations_mut(&mut self) -> &mut HashMap<String, String>;
    fn creation_timestamp_mut(&mut self) -> &mut DateTime<Utc>;
    fn resource_version_mut(&mut self) -> &mut String;
    fn generation_mut(&mut self) -> &mut u64;
    fn deletion_timestamp_mut(&mut self) -> &mut Option<DateTime<Utc>>;
    fn finalizers_mut(&mut self) -> &mut Vec<String>;

    /// ensures that the finalizer is set
    ///
    /// Returns `true` if the finalizer was added and the resource must be stored
    fn ensure_finalizer(&mut self, finalizer: &str) -> bool {
        let finalizers = self.finalizers_mut();
        if !finalizers.iter().any(|r| r == finalizer) {
            finalizers.push(finalizer.into());
            true
        } else {
            false
        }
    }

    /// remove the finalizer from the list of finalizers
    ///
    /// Returns `true` if the finalizer was present before.
    fn remove_finalizer(&mut self, finalizer: &str) -> bool {
        let mut found = false;
        self.finalizers_mut().retain(|f| match f == finalizer {
            true => {
                found = true;
                false
            }
            false => true,
        });
        found
    }
}

pub trait CommonMetadataExt {
    /// Check if a label is present
    fn has_label<L: AsRef<str>>(&self, label: L) -> bool;
    /// Check if a label is present and "true"
    fn has_label_flag<L: AsRef<str>>(&self, label: L) -> bool;
}

impl<C: CommonMetadata> CommonMetadataExt for C {
    fn has_label<L: AsRef<str>>(&self, label: L) -> bool {
        self.labels().contains_key(label.as_ref())
    }

    fn has_label_flag<L: AsRef<str>>(&self, label: L) -> bool {
        self.labels()
            .get(label.as_ref())
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }
}

macro_rules! common_metadata {
    ($name:ty) => {
        impl CommonMetadata for $name {
            fn name(&self) -> &String {
                &self.name
            }
            fn uid(&self) -> &String {
                &self.uid
            }
            fn labels(&self) -> &HashMap<String, String> {
                &self.labels
            }
            fn annotations(&self) -> &HashMap<String, String> {
                &self.annotations
            }
            fn creation_timestamp(&self) -> &DateTime<chrono::Utc> {
                &self.creation_timestamp
            }
            fn resource_version(&self) -> &String {
                &self.resource_version
            }
            fn generation(&self) -> u64 {
                self.generation
            }
            fn deletion_timestamp(&self) -> &Option<DateTime<chrono::Utc>> {
                &self.deletion_timestamp
            }
            fn finalizers(&self) -> &Vec<String> {
                &self.finalizers
            }
        }

        impl CommonMetadataMut for $name {
            fn set_name(&mut self, name: String) {
                self.name = name;
            }
            fn set_uid(&mut self, uid: String) {
                self.uid = uid;
            }
            fn set_labels(&mut self, labels: HashMap<String, String>) {
                self.labels = labels;
            }
            fn set_annotations(&mut self, annotations: HashMap<String, String>) {
                self.annotations = annotations;
            }
            fn set_creation_timestamp(&mut self, creation_timestamp: DateTime<Utc>) {
                self.creation_timestamp = creation_timestamp;
            }
            fn set_resource_version(&mut self, resource_version: String) {
                self.resource_version = resource_version;
            }
            fn set_generation(&mut self, generation: u64) {
                self.generation = generation;
            }
            fn set_deletion_timestamp(&mut self, deletion_timestamp: Option<DateTime<Utc>>) {
                self.deletion_timestamp = deletion_timestamp;
            }
            fn set_finalizers(&mut self, finalizers: Vec<String>) {
                self.finalizers = finalizers;
            }

            fn name_mut(&mut self) -> &mut String {
                &mut self.name
            }
            fn uid_mut(&mut self) -> &mut String {
                &mut self.uid
            }
            fn labels_mut(&mut self) -> &mut HashMap<String, String> {
                &mut self.labels
            }
            fn annotations_mut(&mut self) -> &mut HashMap<String, String> {
                &mut self.annotations
            }
            fn creation_timestamp_mut(&mut self) -> &mut DateTime<chrono::Utc> {
                &mut self.creation_timestamp
            }
            fn resource_version_mut(&mut self) -> &mut String {
                &mut self.resource_version
            }
            fn generation_mut(&mut self) -> &mut u64 {
                &mut self.generation
            }
            fn deletion_timestamp_mut(&mut self) -> &mut Option<DateTime<chrono::Utc>> {
                &mut self.deletion_timestamp
            }
            fn finalizers_mut(&mut self) -> &mut Vec<String> {
                &mut self.finalizers
            }
        }
    };
}

common_metadata!(ScopedMetadata);
common_metadata!(NonScopedMetadata);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set() {
        let mut meta = ScopedMetadata::default();
        let meta_ref: &mut dyn CommonMetadataMut = &mut meta;

        let ts = Utc::now();
        let mut labels = HashMap::new();
        labels.insert("foo".to_string(), "bar".to_string());

        // meta_ref.set_name("foo".into());
        meta_ref.set_name("foo".into());
        meta_ref.set_deletion_timestamp(Some(ts.clone()));
        meta_ref.set_labels(labels.clone());

        assert_eq!(meta.name, "foo");
        assert_eq!(meta.deletion_timestamp, Some(ts));
        assert_eq!(meta.labels, labels);
    }

    #[test]
    fn test_finalizer() {
        let mut meta = ScopedMetadata::default();
        meta.ensure_finalizer("Foo");
        assert_eq!(meta.finalizers, vec!["Foo"]);
        {
            let meta_ref: &mut dyn CommonMetadataMut = &mut meta;
            meta_ref.ensure_finalizer("Bar");
        }
        assert_eq!(meta.finalizers, vec!["Foo".to_string(), "Bar".to_string()]);
        {
            let meta_ref: &mut dyn CommonMetadataMut = &mut meta;
            assert!(!meta_ref.remove_finalizer("Abc"));
            assert!(meta_ref.remove_finalizer("Foo"));
        }
        assert_eq!(meta.finalizers, vec!["Bar".to_string()]);
    }
}
