use super::*;
use serde_json::json;

#[derive(Debug)]
struct ApplicationTestWrapper(Application);

/// We simply don't look at the creation_timestamp and deletion_timestamp of the application
/// because they cannot be created at the exact same time during tests.
impl PartialEq for ApplicationTestWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.spec == other.0.spec
            && self.0.status == other.0.status
            && self.0.metadata.name == other.0.metadata.name
            && self.0.metadata.uid == other.0.metadata.uid
            && self.0.metadata.generation == other.0.metadata.generation
            && self.0.metadata.resource_version == other.0.metadata.resource_version
            && self.0.metadata.finalizers == other.0.metadata.finalizers
            && self.0.metadata.labels == other.0.metadata.labels
            && self.0.metadata.annotations == other.0.metadata.annotations
    }
}

#[test]
fn create_empty_application() {
    let json_app: Application = serde_json::from_value(json!({
    "metadata": {
        "name": "foo",
    },
    "spec": {}
    }))
    .unwrap();

    let app = Application::new("foo");

    assert_eq!(
        ApplicationTestWrapper { 0: app },
        ApplicationTestWrapper { 0: json_app }
    );
}

#[test]
fn create_add_cert() {
    const CERT: &str =
        "LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tDQpNSUlCb2pDQ0FVaWdBxVUsBxTlAraRS0tLS0tDQo=";

    let mut app = Application::new("foo");

    let anchors = app.section::<ApplicationSpecTrustAnchors>();
    assert!(anchors.is_none());

    let anchor = ApplicationSpecTrustAnchorEntry {
        certificate: CERT.into(),
    };
    app.add_trust_anchor(anchor.clone()).unwrap();

    let anchors = app.section::<ApplicationSpecTrustAnchors>();
    assert!(anchors.is_some());

    let anchor_extracted = anchors.unwrap().unwrap();
    assert!(!anchor_extracted.anchors.is_empty());
    assert_eq!(anchor_extracted.anchors[0], anchor);
}
