use crate::{dialect, serde::is_default};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishSpec {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
}

dialect!(PublishSpec[crate::Section::Spec => "publish"]);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSpec {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<Rule>,
}

dialect!(CommandSpec[crate::Section::Spec => "command"]);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    #[serde(default)]
    pub when: When,
    #[serde(default)]
    pub then: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum When {
    Always,
    IsChannel(String),
    Not(Box<When>),
    And(Vec<When>),
    Or(Vec<When>),
}

impl Default for When {
    fn default() -> Self {
        Self::Always
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Step {
    /// Drop the event.
    Drop,
    /// Reject the event.
    Reject(String),
    /// Stop processing and accept the event.
    Break,
    /// Set (replace or add) a cloud events attribute.
    SetAttribute { name: String, value: String },
    /// Remove a cloud events attribute. Ensure that you don't remove a require one.
    RemoveAttribute(String),
    /// Set (replace or add) an extension.
    SetExtension { name: String, value: String },
    /// Remove an extension.
    RemoveExtension(String),
    /// Validate the event using an external endpoint.
    Validate(ValidateSpec),
    /// Enrich the event using an external endpoint.
    Enrich(EnrichSpec),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichSpec {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub request: RequestType,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub response: ResponseType,
    pub endpoint: ExternalEndpoint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateSpec {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub request: RequestType,
    pub endpoint: ExternalEndpoint,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum RequestType {
    /// Send a cloud event
    CloudEvent {
        #[serde(default)]
        #[serde(skip_serializing_if = "is_default")]
        mode: ContentMode,
    },
}

impl Default for RequestType {
    fn default() -> Self {
        Self::CloudEvent {
            mode: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentMode {
    Binary,
    Structured,
}

impl Default for ContentMode {
    fn default() -> Self {
        Self::Binary
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ResponseType {
    /// Expect a cloud event, fails otherwise.
    CloudEvent,
    /// Only consume payload, keep metadata, except content-type.
    Raw,
}

impl Default for ResponseType {
    fn default() -> Self {
        Self::CloudEvent
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalEndpoint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    pub url: String,
    #[serde(default)]
    pub tls: Option<TlsOptions>,
    #[serde(default)]
    pub auth: Authentication,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub timeout: Option<Duration>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TlsOptions {
    #[serde(default)]
    pub insecure: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub certificate: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Authentication {
    None,
    Basic {
        username: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    Bearer {
        token: String,
    },
}

impl Default for Authentication {
    fn default() -> Self {
        Self::None
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_json::json;

    #[test]
    fn default_empty() {
        let spec: PublishSpec = serde_json::from_value(json!({})).unwrap();
        assert_eq!(PublishSpec::default(), spec);
    }

    #[test]
    fn example1() {
        let spec: PublishSpec = serde_json::from_value(json!({
            "rules":[
                {
                    "when": {
                        "and": [
                            { "isChannel": "chan1" },
                            { "not": {
                                "or" : [
                                    { "isChannel": "chan2" },
                                    { "isChannel": "chan3" },
                                ],
                            } },
                        ]
                    },
                    "then": [
                        {
                            "setExtension": {
                                "name": "ext1",
                                "value": "value1",
                            },
                        },
                        {
                            "removeExtension": "ext2",
                        },
                    ],
                }
            ]
        }))
        .unwrap();
        assert_eq!(
            PublishSpec {
                rules: vec![Rule {
                    when: When::And(vec![
                        When::IsChannel("chan1".to_string()),
                        When::Not(Box::new(When::Or(vec![
                            When::IsChannel("chan2".to_string()),
                            When::IsChannel("chan3".to_string()),
                        ])))
                    ]),
                    then: vec![
                        Step::SetExtension {
                            name: "ext1".to_string(),
                            value: "value1".to_string()
                        },
                        Step::RemoveExtension("ext2".to_string()),
                    ],
                }],
            },
            spec
        );
    }

    #[test]
    fn test_deser_1() {
        assert_eq!(
            EnrichSpec {
                request: Default::default(),
                response: Default::default(),
                endpoint: ExternalEndpoint {
                    method: None,
                    url: "http://localhost:1234".to_string(),
                    tls: None,
                    auth: Default::default(),
                    headers: vec![],
                    timeout: None
                }
            },
            serde_json::from_value(json!({
                "request": {
                    "type": "cloudEvent",
                },
                "response": {
                    "type": "cloudEvent",
                },
                "endpoint": {
                    "url": "http://localhost:1234"
                }
            }))
            .unwrap()
        );
    }

    #[test]
    fn test_deser_2() {
        assert_eq!(
            EnrichSpec {
                request: RequestType::CloudEvent {
                    mode: ContentMode::Structured
                },
                response: Default::default(),
                endpoint: ExternalEndpoint {
                    method: None,
                    url: "http://localhost:1234".to_string(),
                    tls: None,
                    auth: Default::default(),
                    headers: vec![],
                    timeout: None
                }
            },
            serde_json::from_value(json!({
                "request": {
                    "type": "cloudEvent",
                    "mode": "structured",
                },
                "response": {
                    "type": "cloudEvent",
                },
                "endpoint": {
                    "url": "http://localhost:1234"
                }
            }))
            .unwrap()
        );
    }
}
