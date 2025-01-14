
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct BaseConfig {
    pub ip: String,
    pub port: Option<u16>,
    pub keys: Keys,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct Keys {
    pub github: String,
    pub travis: Option<String>,
    pub packet_type: PacketType,
    pub endpoint: Vec<url::Url>,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    pub recipient: Option<Recv>,
    pub number: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum PacketType {
    #[default]
    Mix,
    Raw,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Recv {}

impl std::str::FromStr for BaseConfig {
    type Err = toml::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}
 
impl ToString for BaseConfig {
    fn to_string(&self) -> String {
        toml::to_string(self).unwrap()
    }
}

pub const DEFAULT_CONFIG_TOML_STR: &str = r#"
ip = "192.168.1.1"
port = 4433

[keys]
github = "00000000000000000"
travis = "11111111111111111"
packet_type = "Mix"
endpoint = ["https://example.com/api/"]
timeout = "60s"
number=10
"#;


#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const TEST_CONFIG: &str = r#"ip = "127.0.0.1"

[keys]
github = "xxxxxxxxxxxxxxxxx"
travis = "yyyyyyyyyyyyyyyyy"
packet_type = "Mix"
endpoint = ["https://example.com/api/"]
timeout = "5s"
number=100
"#;

    #[test]
    fn deserialize() {
        let config: BaseConfig = toml::from_str(TEST_CONFIG).unwrap();
        assert_eq!(config.ip, "127.0.0.1");
        assert_eq!(config.port, None);
        assert_eq!(config.keys.github, "xxxxxxxxxxxxxxxxx");
        assert_eq!(config.keys.travis.as_ref().unwrap(), "yyyyyyyyyyyyyyyyy");

        let config = BaseConfig::from_str(TEST_CONFIG).unwrap();
        assert_eq!(config.ip, "127.0.0.1");
        assert_eq!(config.port, None);
        assert_eq!(config.keys.github, "xxxxxxxxxxxxxxxxx");
        assert_eq!(config.keys.travis.as_ref().unwrap(), "yyyyyyyyyyyyyyyyy");
    }

    #[test]
    fn serialize() {
        let config = BaseConfig {
            ip: "127.0.0.1".to_string(),
            port: None,
            keys: Keys {
                github: "xxxxxxxxxxxxxxxxx".to_string(),
                travis: Some("yyyyyyyyyyyyyyyyy".to_string()),

                ..Default::default()
            },
        };

        let serialized_toml = toml::to_string(&config).unwrap();
        assert_eq!(serialized_toml, TEST_CONFIG);

        let serialized_toml = config.to_string();
        assert_eq!(serialized_toml, TEST_CONFIG);
    }
}
