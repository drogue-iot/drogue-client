use crate::{Dialect, Section};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub last_transition_time: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default = "default_condition_status")]
    pub status: String,
    pub r#type: String,
}

#[derive(Clone, Debug, Default)]
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

impl Conditions {
    fn make_status(status: Option<bool>) -> String {
        match status {
            Some(true) => "True",
            Some(false) => "False",
            None => "Unknown",
        }
        .into()
    }

    pub fn update<S>(&mut self, r#type: S, status: ConditionStatus)
    where
        S: AsRef<str>,
    {
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
}
