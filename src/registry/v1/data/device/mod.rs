use crate::{
    attribute, dialect, meta::v1::ScopedMetadata, serde::is_default, translator, Dialect, Section,
    Translator,
};
use core::fmt::{self, Formatter};
use serde::{de::MapAccess, Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// A device.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct Device {
    pub metadata: ScopedMetadata,
    #[serde(default)]
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub spec: Map<String, Value>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub status: Map<String, Value>,
}

translator!(Device);

impl Device {
    /// Validate if a device is enabled
    pub fn validate_device(&self) -> bool {
        match self.section::<DeviceSpecCore>() {
            // found "core", decoded successfully -> check
            Some(Ok(core)) => {
                if core.disabled {
                    return false;
                }
            }
            // found "core", but could not decode -> fail
            Some(Err(_)) => {
                return false;
            }
            // no "core" section
            _ => {}
        };

        // done
        true
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct DeviceSpecCore {
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub disabled: bool,
}

attribute!(pub DeviceSpecCore[DeviceEnabled:bool] => |core| match core {
    Some(Ok(core)) => core.disabled,
    // failed to decode
    Some(Err(_)) => false,
    // no "core" section
    None => true,
});
attribute!(pub DeviceSpecCommands[Commands:Vec<Command>] => |commands| match commands {
    Some(Ok(commands)) => commands.commands,
    _ => vec![],
});
attribute!(pub DeviceSpecCommands[FirstCommand:Option<Command>] => |commands| match commands {
    Some(Ok(commands)) => commands.commands.get(0).cloned(),
    _ => None,
});

impl Dialect for DeviceSpecCore {
    fn key() -> &'static str {
        "core"
    }
    fn section() -> Section {
        Section::Spec
    }
}

/// Configured device credentials.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct DeviceSpecCredentials {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub credentials: Vec<Credential>,
}

impl Dialect for DeviceSpecCredentials {
    fn key() -> &'static str {
        "credentials"
    }
    fn section() -> Section {
        Section::Spec
    }
}

/// A single credential entry.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Credential {
    #[serde(rename = "user")]
    UsernamePassword {
        username: String,
        password: Password,
        #[serde(default)]
        unique: bool,
    },
    #[serde(rename = "pass")]
    Password(Password),
    #[serde(rename = "cert")]
    Certificate(String),
}

#[derive(Clone, Serialize, PartialEq, Eq)]
pub enum Password {
    #[serde(rename = "plain")]
    Plain(String),
    #[serde(rename = "bcrypt")]
    BCrypt(String),
    #[serde(rename = "sha512")]
    Sha512(String),
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PasswordVisitor)
    }
}

struct PasswordVisitor;

impl<'de> serde::de::Visitor<'de> for PasswordVisitor {
    type Value = Password;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("A password, by string or map")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Password::Plain(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Password::Plain(value))
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        if let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "plain" => Ok(Password::Plain(map.next_value()?)),
                "bcrypt" => Ok(Password::BCrypt(map.next_value()?)),
                "sha512" => Ok(Password::Sha512(map.next_value()?)),
                key => Err(serde::de::Error::unknown_field(
                    key,
                    &["plain", "bcrypt", "sha512"],
                )),
            }
        } else {
            Err(serde::de::Error::invalid_length(
                0,
                &"Expected exactly one field",
            ))
        }
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("...")
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSpecGatewaySelector {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub match_names: Vec<String>,
}

impl Dialect for DeviceSpecGatewaySelector {
    fn key() -> &'static str {
        "gatewaySelector"
    }
    fn section() -> Section {
        Section::Spec
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct DeviceSpecCommands {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<Command>,
}

impl Dialect for DeviceSpecCommands {
    fn key() -> &'static str {
        "commands"
    }
    fn section() -> Section {
        Section::Spec
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Command {
    #[serde(rename = "external")]
    External(ExternalEndpoint),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExternalEndpoint {
    pub r#type: Option<String>,
    pub url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub method: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct DeviceSpecAliases(
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub Vec<String>,
);

dialect!(DeviceSpecAliases [Section::Spec => "alias"]);

#[cfg(test)]
mod test {

    use super::*;
    use serde_json::json;

    #[test]
    fn deser_credentials_legacy_plain() {
        let des = serde_json::from_value::<Vec<Credential>>(json! {[
            {"pass": "foo"},
            {"user": {"username": "foo", "password": "bar"}}
        ]});
        assert_eq!(
            des.unwrap(),
            vec![
                Credential::Password(Password::Plain("foo".into())),
                Credential::UsernamePassword {
                    username: "foo".into(),
                    password: Password::Plain("bar".into()),
                    unique: false,
                },
            ]
        )
    }

    #[test]
    fn deser_aliases() {
        let des = serde_json::from_value::<DeviceSpecAliases>(json!(["drogue", "iot"]));
        assert_eq!(des.unwrap().0, vec!["drogue", "iot"]);
    }
}
