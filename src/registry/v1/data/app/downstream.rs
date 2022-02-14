use crate::{dialect, Section};
use serde::{Deserialize, Serialize};

dialect!(DownstreamSpec [Section::Spec => "downstream"]);

/// The application downstream specification.
#[derive(Clone, Default, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DownstreamSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[cfg(test)]
mod test {
    use crate::{
        registry::v1::{Application, DownstreamSpec},
        Translator,
    };
    use serde_json::json;

    #[test]
    fn test_json_default() {
        let mut app = Application::default();
        app.set_section(DownstreamSpec::default()).unwrap();
        let json = serde_json::to_value(&app).unwrap();
        assert_eq!(json!({"downstream": {}}), json["spec"]);
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
            Default::default()
        );
    }

    #[test]
    fn test_deserialize_empty() {
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
        assert_eq!(spec.transpose().unwrap(), Some(Default::default()));
    }

    #[test]
    fn test_deserialize_password() {
        let app: Application = serde_json::from_value(json!({
            "metadata": {
                "name": "foo",
            },
            "spec": {
                "downstream": {
                    "password": "foobar",
                },
            }
        }))
        .unwrap();

        let spec = app.section::<DownstreamSpec>();
        assert_eq!(
            spec.transpose().unwrap(),
            Some(DownstreamSpec {
                password: Some("foobar".to_string())
            })
        );
    }
}
