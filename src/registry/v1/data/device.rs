use crate::{
    attribute, meta::v1::ScopedMetadata, serde::is_default, translator, Dialect, Section,
    Translator,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
    Some(Ok(commands)) => commands.commands.clone(),
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
        password: String,
        #[serde(default)]
        unique: bool,
    },
    #[serde(rename = "pass")]
    Password(String),
    #[serde(rename = "cert")]
    Certificate(String),
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
    pub endpoint: String,
    pub r#type: Option<String>,
}
