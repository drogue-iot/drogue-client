use crate::{Dialect, Section};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

pub const CONDITION_READY: &str = "Ready";

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub last_transition_time: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "is_empty")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "is_empty")]
    pub reason: Option<String>,
    #[serde(default = "default_condition_status")]
    pub status: String,
    pub r#type: String,
}

fn is_empty(value: &Option<String>) -> bool {
    match value {
        None => true,
        Some(str) if str.is_empty() => true,
        _ => false,
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Conditions(pub Vec<Condition>);

fn default_condition_status() -> String {
    "Unknown".into()
}

impl Dialect for Conditions {
    fn key() -> &'static str {
        "conditions"
    }

    fn section() -> Section {
        Section::Status
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConditionStatus {
    pub status: Option<bool>,
    pub reason: Option<String>,
    pub message: Option<String>,
}

impl From<Option<bool>> for ConditionStatus {
    fn from(value: Option<bool>) -> Self {
        Self {
            status: value,
            ..Default::default()
        }
    }
}

impl From<bool> for ConditionStatus {
    fn from(value: bool) -> Self {
        Self {
            status: Some(value),
            ..Default::default()
        }
    }
}

impl Conditions {
    fn make_status(status: Option<bool>) -> String {
        match status {
            Some(true) => "True",
            Some(false) => "False",
            None => "Unknown",
        }
        .into()
    }

    pub fn update<T, S>(&mut self, r#type: T, status: S)
    where
        T: AsRef<str>,
        S: Into<ConditionStatus>,
    {
        let status = status.into();
        let str_status = Self::make_status(status.status);

        for mut condition in &mut self.0 {
            if condition.r#type == r#type.as_ref() {
                if condition.status != str_status {
                    condition.last_transition_time = Utc::now();
                    condition.status = str_status;
                }
                condition.reason = status.reason;
                condition.message = status.message;

                return;
            }
        }

        // did not find entry so far

        self.0.push(Condition {
            last_transition_time: Utc::now(),
            message: status.message,
            reason: status.reason,
            status: str_status,
            r#type: r#type.as_ref().into(),
        });
    }

    /// Aggregate the "Ready" condition.
    pub fn aggregate_ready(mut self) -> Self {
        let mut ready = true;
        for condition in &self.0 {
            if condition.r#type == CONDITION_READY {
                continue;
            }

            if condition.status != "True" {
                ready = false;
                break;
            }
        }

        self.update(
            CONDITION_READY,
            match ready {
                true => ConditionStatus {
                    status: Some(true),
                    reason: None,
                    message: None,
                },
                false => ConditionStatus {
                    status: Some(false),
                    reason: Some("NonReadyConditions".into()),
                    message: None,
                },
            },
        );
        self
    }

    /// Clear the provided condition and re-aggregate the ready state.
    pub fn clear_ready<T>(mut self, r#type: T) -> Self
    where
        T: AsRef<str>,
    {
        let r#type = r#type.as_ref();
        self.0.retain(|c| c.r#type != r#type);
        self.aggregate_ready()
    }
}

impl Deref for Conditions {
    type Target = Vec<Condition>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Conditions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn insert_cond_1() {
        let mut conditions = Conditions::default();

        conditions.update(
            "KafkaReady",
            ConditionStatus {
                status: Some(true),
                ..Default::default()
            },
        );

        let now = Utc::now();
        conditions.0[0].last_transition_time = now;

        assert_eq!(
            conditions.0,
            vec![Condition {
                last_transition_time: now,
                message: None,
                reason: None,
                status: "True".to_string(),
                r#type: "KafkaReady".to_string()
            }]
        );
    }

    #[test]
    fn update_cond_1() {
        let mut conditions = Conditions::default();

        // create two conditions

        conditions.update(
            "KafkaReady",
            ConditionStatus {
                status: Some(true),
                ..Default::default()
            },
        );
        conditions.update(
            "FooBarReady",
            ConditionStatus {
                status: None,
                ..Default::default()
            },
        );

        // reset timestamps to known values
        let now = Utc::now();
        conditions.0[0].last_transition_time = now;
        conditions.0[1].last_transition_time = now;

        assert_eq!(
            conditions.0,
            vec![
                Condition {
                    last_transition_time: now,
                    message: None,
                    reason: None,
                    status: "True".to_string(),
                    r#type: "KafkaReady".to_string()
                },
                Condition {
                    last_transition_time: now,
                    message: None,
                    reason: None,
                    status: "Unknown".to_string(),
                    r#type: "FooBarReady".to_string()
                }
            ]
        );

        conditions.update(
            "FooBarReady",
            ConditionStatus {
                status: Some(true),
                message: Some("All systems are ready to go".into()),
                reason: Some("AllSystemsGo".into()),
                ..Default::default()
            },
        );

        // the second timestamp should be different now
        assert_eq!(conditions.0[0].last_transition_time, now);
        assert_ne!(conditions.0[1].last_transition_time, now);

        // reset timestamps to known values
        let now = Utc::now();
        conditions.0[0].last_transition_time = now;
        conditions.0[1].last_transition_time = now;

        assert_eq!(
            conditions.0,
            vec![
                Condition {
                    last_transition_time: now,
                    message: None,
                    reason: None,
                    status: "True".to_string(),
                    r#type: "KafkaReady".to_string()
                },
                Condition {
                    last_transition_time: now,
                    message: Some("All systems are ready to go".into()),
                    reason: Some("AllSystemsGo".into()),
                    status: "True".to_string(),
                    r#type: "FooBarReady".to_string()
                }
            ]
        );
    }

    #[test]
    fn serialize() {
        let json = serde_json::to_string(&Conditions(vec![Condition {
            last_transition_time: DateTime::parse_from_rfc3339("2001-02-03T12:34:56Z")
                .expect("Valid timestmap")
                .with_timezone(&Utc),
            message: None,
            reason: None,
            status: "True".to_string(),
            r#type: "Ready".to_string(),
        }]))
        .expect("Serialize to JSON");
        assert_eq!(
            json,
            r#"[{"lastTransitionTime":"2001-02-03T12:34:56Z","status":"True","type":"Ready"}]"#
        );
    }

    #[test]
    fn conversions() {
        let mut conditions = Conditions::default();
        conditions.update(
            "Foo",
            ConditionStatus {
                status: None,
                reason: None,
                message: None,
            },
        );
        conditions.update("Foo", true);
        conditions.update("Bar", Some(true));
    }

    #[test]
    fn clear_ready() {
        let mut conditions = Conditions::default();
        conditions.update(
            "SomeReady",
            ConditionStatus {
                status: None,
                reason: None,
                message: None,
            },
        );
        conditions = conditions.aggregate_ready();
        assert_eq!(conditions.len(), 2);
        conditions = conditions.clear_ready("SomeReady");
        assert_eq!(conditions.len(), 1);
        let c = conditions.get(0).unwrap();
        assert_eq!(c.r#type, "Ready");
        assert_eq!(c.status, "True");
    }
}
