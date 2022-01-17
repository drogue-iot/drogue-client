use crate::dialect;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishSpec {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<PublishRule>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishRule {
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
    SetHeader { name: String, value: String },
    RemoveHeader(String),
}

dialect!(PublishSpec[crate::Section::Spec => "publish"]);

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
                    "then":  [
                        {
                            "setHeader": {
                                "name": "header1",
                                "value": "value1",
                            },
                        },
                        {
                            "removeHeader": "header2",
                        },
                    ],
                }
            ]
        }))
        .unwrap();
        assert_eq!(
            PublishSpec {
                rules: vec![PublishRule {
                    when: When::And(vec![
                        When::IsChannel("chan1".to_string()),
                        When::Not(Box::new(When::Or(vec![
                            When::IsChannel("chan2".to_string()),
                            When::IsChannel("chan3".to_string()),
                        ])))
                    ]),
                    then: vec![
                        Step::SetHeader {
                            name: "header1".to_string(),
                            value: "value1".to_string()
                        },
                        Step::RemoveHeader("header2".to_string()),
                    ],
                }],
            },
            spec
        );
    }
}
