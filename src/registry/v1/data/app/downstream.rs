use crate::{dialect, Section};
use base64_serde::Deserializer;
use core::fmt;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Serialize, Serializer};
use std::collections::HashMap;

dialect!(DownstreamSpec [Section::Spec => "downstream"]);

/// The application downstream specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DownstreamSpec {
    ExternalKafka(ExternalKafkaSpec),
    Internal(InternalSpec),
}

impl Serialize for DownstreamSpec {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = s.serialize_struct("externalKafka", 0)?;
        match self {
            Self::ExternalKafka(kafka) => {
                s.serialize_field("externalKafka", &kafka)?;
            }
            Self::Internal(kafka) => {
                if kafka != &Default::default() {
                    s.serialize_field("internal", &kafka)?;
                }
            }
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for DownstreamSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Variant {
            Internal,
            ExternalKafka,
        }

        impl<'de> Deserialize<'de> for Variant {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct VariantVisitor;

                impl<'de> Visitor<'de> for VariantVisitor {
                    type Value = Variant;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("Variant type")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Variant, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "externalKafka" => Ok(Variant::ExternalKafka),
                            _ => Ok(Variant::Internal),
                        }
                    }
                }

                deserializer.deserialize_identifier(VariantVisitor)
            }
        }

        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = DownstreamSpec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("DownstreamSpec variant")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                match map.next_key()? {
                    Some(Variant::ExternalKafka) => {
                        Ok(DownstreamSpec::ExternalKafka(map.next_value()?))
                    }
                    Some(Variant::Internal) => Ok(DownstreamSpec::Internal(map.next_value()?)),
                    None => Ok(DownstreamSpec::Internal(Default::default())),
                }
            }
        }

        deserializer.deserialize_map(ValueVisitor)
    }
}

/// Defaulting to the internally managed downstream target.
impl Default for DownstreamSpec {
    fn default() -> Self {
        Self::Internal(Default::default())
    }
}

/// The downstream specification when using externally provided Kafka.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalKafkaSpec {
    pub bootstrap_servers: String,
    pub topic: String,
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// The downstream specification when using internally managed resources.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InternalSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[cfg(test)]
mod test {
    use crate::registry::v1::{ExternalKafkaSpec, InternalSpec};
    use crate::{
        registry::v1::{Application, DownstreamSpec},
        Translator,
    };
    use maplit::{convert_args, hashmap};
    use serde_json::json;

    #[test]
    fn test_json_default() {
        let mut app = Application::default();
        app.set_section(DownstreamSpec::Internal(Default::default()))
            .unwrap();
        let json = serde_json::to_value(&app).unwrap();
        assert_eq!(json!({"downstream": {}}), json["spec"]);
    }

    #[test]
    fn test_json_external_kafka() {
        let mut app = Application::default();
        app.set_section(DownstreamSpec::ExternalKafka(ExternalKafkaSpec {
            bootstrap_servers: "server1:9091".into(),
            topic: "topic-1".into(),
            properties: convert_args!(hashmap!(
                "foo.bar" => "baz",
            )),
        }))
        .unwrap();
        let json = serde_json::to_value(&app).unwrap();
        assert_eq!(
            json!({
                "downstream": {
                    "externalKafka": {
                        "bootstrapServers": "server1:9091",
                        "topic": "topic-1",
                        "properties": {
                            "foo.bar": "baz",
                        }
                    }
                }
            }),
            json["spec"]
        );
    }

    #[test]
    fn test_deserialize_none() {
        let app: Application = serde_json::from_value(json!({
            "metadata": {
                "name": "foo",
            }
        }))
        .unwrap();

        let spec = app.section::<DownstreamSpec>();
        assert!(spec.is_none());
        assert_eq!(
            spec.transpose().unwrap().unwrap_or_default(),
            DownstreamSpec::Internal(Default::default())
        );
    }

    #[test]
    fn test_deserialize_unknown() {
        let app: Application = serde_json::from_value(json!({
            "metadata": {
                "name": "foo",
            },
            "spec": {
                "downstream": { "foo": {} },
            }
        }))
        .unwrap();

        let spec = app.section::<DownstreamSpec>();
        assert_eq!(
            spec.transpose().unwrap(),
            Some(DownstreamSpec::Internal(Default::default()))
        );
    }

    #[test]
    fn test_deserialize_internal() {
        let app: Application = serde_json::from_value(json!({
            "metadata": {
                "name": "foo",
            },
            "spec": {
                "downstream": {},
            }
        }))
        .unwrap();

        let spec = app.section::<DownstreamSpec>();
        assert_eq!(
            spec.transpose().unwrap(),
            Some(DownstreamSpec::Internal(Default::default()))
        );
    }

    #[test]
    fn test_deserialize_internal_2() {
        let app: Application = serde_json::from_value(json!({
            "metadata": {
                "name": "foo",
            },
            "spec": {
                "downstream": {
                    "internal": {}
                },
            }
        }))
        .unwrap();

        let spec = app.section::<DownstreamSpec>();
        assert_eq!(
            spec.transpose().unwrap(),
            Some(DownstreamSpec::Internal(Default::default()))
        );
    }

    #[test]
    fn test_deserialize_internal_password() {
        let app: Application = serde_json::from_value(json!({
            "metadata": {
                "name": "foo",
            },
            "spec": {
                "downstream": {
                    "internal": {
                        "password": "foobar",
                    }
                },
            }
        }))
        .unwrap();

        let spec = app.section::<DownstreamSpec>();
        assert_eq!(
            spec.transpose().unwrap(),
            Some(DownstreamSpec::Internal(InternalSpec {
                password: Some("foobar".to_string())
            }))
        );
    }

    #[test]
    fn test_deserialize_external() {
        let app: Application = serde_json::from_value(json!({
            "metadata": {
                "name": "foo",
            },
            "spec": {
                "downstream": {
                    "externalKafka": {
                        "bootstrapServers": "server",
                        "topic": "topic",
                    }
                },
            }
        }))
        .unwrap();

        let spec = app.section::<DownstreamSpec>();
        assert_eq!(
            spec.transpose().unwrap(),
            Some(DownstreamSpec::ExternalKafka(ExternalKafkaSpec {
                bootstrap_servers: "server".into(),
                topic: "topic".into(),
                properties: Default::default()
            }))
        );
    }
}
