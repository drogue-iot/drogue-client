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
    PlainTopic {
        #[serde(default, skip_serializing_if = "is_default")]
        device_prefix: bool,
    },
    #[serde(rename_all = "camelCase")]
    #[serde(alias = "wot")]
    WebOfThings {
        #[serde(default, skip_serializing_if = "is_default")]
        node_wot_bug: bool,
    },
    #[serde(rename_all = "camelCase")]
    #[serde(alias = "c8y")]
    Cumulocity,
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
    fn test_plain_default() {
        assert_eq!(
            MqttSpec {
                dialect: MqttDialect::PlainTopic {
                    device_prefix: false
                }
            },
            serde_json::from_value(json!({
                "dialect":{
                    "type": "plainTopic",
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

    #[test]
    fn test_wot() {
        assert_eq!(
            MqttSpec {
                dialect: MqttDialect::WebOfThings {
                    node_wot_bug: false,
                }
            },
            serde_json::from_value(json!({
                "dialect":{
                    "type": "wot",
                }
            }))
            .unwrap()
        )
    }

    #[test]
    fn test_wot_bug() {
        assert_eq!(
            MqttSpec {
                dialect: MqttDialect::WebOfThings { node_wot_bug: true }
            },
            serde_json::from_value(json!({
                "dialect":{
                    "type": "wot",
                    "nodeWotBug": true,
                }
            }))
            .unwrap()
        )
    }
}
