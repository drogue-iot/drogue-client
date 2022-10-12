use crate::{
    attribute, dialect,
    meta::v1::{CommonMetadata, CommonMetadataMut, ScopedMetadata},
    serde::{is_default, Base64Standard},
    translator, Dialect, Section, Translator,
};
use chrono::{DateTime, Utc};
use core::fmt::{self, Formatter};
use serde::{de::MapAccess, Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use std::{cmp::Ordering, collections::HashMap};

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

impl AsRef<dyn CommonMetadata> for Device {
    fn as_ref(&self) -> &(dyn CommonMetadata + 'static) {
        &self.metadata
    }
}

impl AsMut<dyn CommonMetadataMut> for Device {
    fn as_mut(&mut self) -> &mut (dyn CommonMetadataMut + 'static) {
        &mut self.metadata
    }
}

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

    /// Create an minimal device object from the an application name and a device name
    pub fn new<A, D>(application: A, device: D) -> Self
    where
        A: AsRef<str>,
        D: AsRef<str>,
    {
        Device {
            metadata: ScopedMetadata {
                application: application.as_ref().into(),
                name: device.as_ref().into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Insert a credential entry to the crendentials of a device.
    /// If there are no credentials already existing an array is created
    /// if there is an error deserializing the existing data an error is returned
    pub fn add_credential(&mut self, credential: Credential) -> Result<(), serde_json::Error> {
        // TODO: Remove before drg version 0.12.x
        self.update_section::<DeviceSpecCredentials, _>(|mut auth| {
            auth.credentials.push(credential.clone());
            auth
        })?;
        self.update_section::<DeviceSpecAuthentication, _>(|mut auth| {
            auth.credentials.push(credential);
            auth
        })
    }

    /// Retrieve the credentials of this device
    pub fn get_credentials(&self) -> Option<Vec<Credential>> {
        let credentials = match self.section::<DeviceSpecAuthentication>() {
            Some(Ok(auth)) => auth.credentials,
            _ => match self.section::<DeviceSpecCredentials>() {
                Some(Ok(credentials)) => credentials.credentials,
                _ => {
                    return None;
                }
            },
        };
        Some(credentials)
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
    #[serde(rename = "psk")]
    PreSharedKey(PreSharedKey),
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

/// Configured device credentials.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct DeviceSpecAuthentication {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub credentials: Vec<Credential>,
}

impl Dialect for DeviceSpecAuthentication {
    fn key() -> &'static str {
        "authentication"
    }
    fn section() -> Section {
        Section::Spec
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreSharedKey {
    #[serde(with = "Base64Standard")]
    pub key: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity: Option<Validity>,
}

impl fmt::Debug for PreSharedKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "key=..., validity: {:?}", self.validity)
    }
}

impl PartialOrd for PreSharedKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.validity.is_none() && other.validity.is_none() {
            Some(Ordering::Equal)
        } else if self.validity.is_none() {
            Some(Ordering::Less)
        } else if other.validity.is_none() {
            Some(Ordering::Equal)
        } else {
            self.validity.partial_cmp(&other.validity)
        }
    }
}

impl Ord for PreSharedKey {
    fn cmp(&self, other: &Self) -> Ordering {
        // We know it never returns None
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Validity {
    #[serde(rename = "notBefore")]
    pub not_before: DateTime<Utc>,
    #[serde(rename = "notAfter")]
    pub not_after: DateTime<Utc>,
}

impl Validity {
    pub fn is_valid(&self, now: DateTime<Utc>) -> bool {
        self.not_before <= now && self.not_after >= now
    }
}

impl PartialOrd for Validity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(if self.not_before < other.not_before {
            Ordering::Less
        } else if self.not_before > other.not_before {
            Ordering::Greater
        } else if self.not_after > other.not_after {
            Ordering::Less
        } else if self.not_after < other.not_after {
            Ordering::Greater
        } else {
            Ordering::Equal
        })
    }
}

impl Ord for Validity {
    fn cmp(&self, other: &Self) -> Ordering {
        // We know it never returns None
        self.partial_cmp(other).unwrap()
    }
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
    External(ExternalCommandEndpoint),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExternalCommandEndpoint {
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
mod test;
