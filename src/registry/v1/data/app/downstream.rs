use crate::{dialect, Dialect, Section};
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
    Internal,
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
            Self::Internal => {}
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
                    _ => Ok(DownstreamSpec::Internal),
                }
            }
        }

        deserializer.deserialize_map(ValueVisitor)
    }
}

/// Defaulting to the internally managed downstream target.
impl Default for DownstreamSpec {
    fn default() -> Self {
        Self::Internal
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

#[cfg(test)]
mod test {
    use crate::registry::v1::ExternalKafkaSpec;
    use crate::{
        registry::v1::{Application, DownstreamSpec},
        Translator,
    };
    use maplit::{convert_args, hashmap};
    use serde_json::json;

    #[test]
    fn test_json_default() {
        let mut app = Application::default();
        app.set_section(DownstreamSpec::Internal).unwrap();
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
            DownstreamSpec::Internal
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
        assert_eq!(spec.transpose().unwrap(), Some(DownstreamSpec::Internal));
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
        assert_eq!(spec.transpose().unwrap(), Some(DownstreamSpec::Internal));
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
