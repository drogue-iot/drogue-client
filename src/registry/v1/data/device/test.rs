use super::*;
use chrono::Duration;
use serde_json::json;

#[derive(Debug)]
struct DeviceTestWrapper(Device);

/// We simply don't look at the creation_timestamp and deletion_timestamp of the devices
/// because they cannot be created at the exact same time during tests.
impl PartialEq for DeviceTestWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.spec == other.0.spec
            && self.0.status == other.0.status
            && self.0.metadata.name == other.0.metadata.name
            && self.0.metadata.application == other.0.metadata.application
            && self.0.metadata.uid == other.0.metadata.uid
            && self.0.metadata.generation == other.0.metadata.generation
            && self.0.metadata.resource_version == other.0.metadata.resource_version
            && self.0.metadata.finalizers == other.0.metadata.finalizers
            && self.0.metadata.labels == other.0.metadata.labels
            && self.0.metadata.annotations == other.0.metadata.annotations
    }
}

#[test]
fn deser_credentials() {
    let des = serde_json::from_value::<Vec<Credential>>(json! {[
        {"pass": "foo"},
        {"pass": {"bcrypt": "$2a$12$/ooOoK.qKkqo2GvCvgt0ae076ak0Aa8VoLTW2Ei/WUgZ2n9kt1zZ2"}},
        {"user": {"username": "foo", "password": "bar"}},
        {"user": {"username": "foo", "password": {"sha512": "$6$ncx1PBP3mqha5Z7B$GXz/Q14oxbGcIx78lJ19Jxnx38v.Dp0zgmprUAWVjv4Y447SmBfUFLtDByZnoIneekTAPHjQS.osdZ3rYWdk/."}}},
        {"psk": {"key": "bWV0YWxsaWNh"}},
        {"psk": {"key": "bWFkcnVnYWRh", "validity": { "notBefore": "2022-10-05T07:05:26Z", "notAfter": "2022-10-06T07:05:26Z" }}}
    ]});
    assert_eq!(
        des.unwrap(),
        vec![
            Credential::Password(Password::Plain("foo".into())),
            Credential::Password(Password::BCrypt(
                "$2a$12$/ooOoK.qKkqo2GvCvgt0ae076ak0Aa8VoLTW2Ei/WUgZ2n9kt1zZ2".into()
            )),
            Credential::UsernamePassword {
                username: "foo".into(),
                password: Password::Plain("bar".into()),
                unique: false,
            },
            Credential::UsernamePassword {
                username: "foo".into(),
                password: Password::Sha512("$6$ncx1PBP3mqha5Z7B$GXz/Q14oxbGcIx78lJ19Jxnx38v.Dp0zgmprUAWVjv4Y447SmBfUFLtDByZnoIneekTAPHjQS.osdZ3rYWdk/.".into()),
                unique: false,
            },
            Credential::PreSharedKey(PreSharedKey {
                key: b"metallica".to_vec(),
                validity: None,

            }),
            Credential::PreSharedKey(PreSharedKey {
                key: b"madrugada".to_vec(),
                validity: Some(Validity {
                    not_before: DateTime::parse_from_rfc3339("2022-10-05T07:05:26Z").unwrap().into(),
                    not_after: DateTime::parse_from_rfc3339("2022-10-06T07:05:26Z").unwrap().into(),
                }),
            }),
        ]
    )
}

#[test]
fn deser_aliases() {
    let des = serde_json::from_value::<DeviceSpecAliases>(json!(["drogue", "iot"]));
    assert_eq!(des.unwrap().0, vec!["drogue", "iot"]);
}

#[test]
fn create_empty_device() {
    let json_device: Device = serde_json::from_value(json!({
    "metadata": {
        "name": "foo",
        "application": "foo_app"
    },
    "spec": {}
    }))
    .unwrap();

    let device = Device::new("foo_app", "foo");

    assert_eq!(
        DeviceTestWrapper { 0: device },
        DeviceTestWrapper { 0: json_device }
    );
}

#[test]
fn create_add_credential() {
    let mut device = Device::new("foo_app", "foo");

    let creds = device.section::<DeviceSpecCredentials>();
    assert!(creds.is_none());

    let creds = device.section::<DeviceSpecAuthentication>();
    assert!(creds.is_none());

    let password = Credential::Password {
        0: Password::Plain("very_secret".into()),
    };
    device.add_credential(password.clone()).unwrap();

    let creds = device.section::<DeviceSpecCredentials>();
    assert!(creds.is_some());

    let creds = device.section::<DeviceSpecAuthentication>();
    assert!(creds.is_some());

    let password_extracted = creds.unwrap().unwrap();
    assert!(!password_extracted.credentials.is_empty());
    assert_eq!(password_extracted.credentials[0], password);
}

#[test]
fn psk_ordering() {
    let base: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;

    let no_validity = PreSharedKey {
        key: b"foo".to_vec(),
        validity: None,
    };

    let oldest = PreSharedKey {
        key: b"foo".to_vec(),
        validity: Some(Validity {
            not_before: base,
            not_after: base + Duration::days(10),
        }),
    };

    let newer = PreSharedKey {
        key: b"foo".to_vec(),
        validity: Some(Validity {
            not_before: base + Duration::days(3),
            not_after: base + Duration::days(11),
        }),
    };

    let newest = PreSharedKey {
        key: b"foo".to_vec(),
        validity: Some(Validity {
            not_before: base + Duration::days(5),
            not_after: base + Duration::days(11),
        }),
    };

    let mut keys = vec![
        newest.clone(),
        newer.clone(),
        no_validity.clone(),
        oldest.clone(),
    ];
    keys.sort();

    assert_eq!(keys[0], no_validity);
    assert_eq!(keys[1], oldest);
    assert_eq!(keys[2], newer);
    assert_eq!(keys[3], newest);
}

#[test]
fn psk_validity() {
    let base: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;

    let validity = Validity {
        not_before: base + Duration::days(5),
        not_after: base + Duration::days(7),
    };

    assert!(!validity.is_valid(DateTime::<Utc>::MIN_UTC + Duration::days(4)));
    assert!(validity.is_valid(DateTime::<Utc>::MIN_UTC + Duration::days(5)));
    assert!(validity.is_valid(DateTime::<Utc>::MIN_UTC + Duration::days(6)));
    assert!(validity.is_valid(DateTime::<Utc>::MIN_UTC + Duration::days(7)));
    assert!(!validity.is_valid(DateTime::<Utc>::MIN_UTC + Duration::days(8)));
}
