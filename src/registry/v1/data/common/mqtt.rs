use crate::{dialect, serde::is_default, Section};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MqttSpec {
    #[serde(default, skip_serializing_if = "is_default")]
    pub dialect: MqttDialect,
}

dialect!(MqttSpec [Section::Spec => "mqtt"]);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum MqttDialect {
    #[serde(rename = "drogue/v1")]
    DrogueV1,
    #[serde(rename_all = "camelCase")]
    PlainTopic { device_prefix: bool },
}

impl Default for MqttDialect {
    fn default() -> Self {
        Self::DrogueV1
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(
            MqttSpec {
                dialect: MqttDialect::DrogueV1
            },
            serde_json::from_value(json!({})).unwrap()
        )
    }

    #[test]
    fn test_explicit_v1() {
        assert_eq!(
            MqttSpec {
                dialect: MqttDialect::DrogueV1
            },
            serde_json::from_value(json!({
                "dialect": {
                    "type": "drogue/v1",
                }
            }))
            .unwrap()
        )
    }

    #[test]
    fn test_plain_true() {
        assert_eq!(
            MqttSpec {
                dialect: MqttDialect::PlainTopic {
                    device_prefix: true
                }
            },
            serde_json::from_value(json!({
                "dialect":{
                    "type": "plainTopic",
                    "devicePrefix": true,
                }
            }))
            .unwrap()
        )
    }
}
