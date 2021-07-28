use crate::{dialect, Dialect, Section};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

dialect!(DownstreamSpec [Section::Spec => "downstream"]);

/// The application downstream specification.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DownstreamSpec {
    ExternalKafka(ExternalKafkaSpec),
    #[serde(other)]
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
}
