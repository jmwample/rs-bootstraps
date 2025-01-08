use config_types::BaseConfig;
use std::str::FromStr;

#[cfg(feature = "enable-cfg")]
include!(concat!(env!("OUT_DIR"), "/obfuscated.rs"));

pub struct Config(BaseConfig);

/// Default Configuration used if no bootstrap configuration file is provided at compile time.
const DEFAULT_CONFIG_STR: &str = r#"
ip = "192.168.1.1"
port = 4433

[keys]
github = "00000000000000000"
travis = "11111111111111111"
"#;

impl AsRef<BaseConfig> for Config {
    fn as_ref(&self) -> &BaseConfig {
        &self.0
    }
}

impl Default for Config {
    fn default() -> Self {
        #[cfg(feature = "require-cfg")]
        let config_str = BOOTSTRAP_CONFIG_STR;

        #[cfg(all(not(feature = "require-cfg"), feature = "enable-cfg"))]
        let config_str = if !BOOTSTRAP_CONFIG_STR.is_empty() {
            BOOTSTRAP_CONFIG_STR
        } else {
            DEFAULT_CONFIG_STR
        };

        #[cfg(not(feature = "enable-cfg"))]
        let config_str = DEFAULT_CONFIG_STR;

        let config = BaseConfig::from_str(config_str).expect("failed to parse default config");

        Self(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        #[cfg(feature = "enable-cfg")]
        {
            println!("{:?}", BOOTSTRAP_CONFIG_STR);
            let c = Config::default();
            let config = c.as_ref();
            assert_eq!(config.ip, "127.0.0.1");
            assert_eq!(config.port, None);
            assert_eq!(config.keys.github, "xxxxxxxxxxxxxxxxx");
            assert_eq!(config.keys.travis.as_ref().unwrap(), "yyyyyyyyyyyyyyyyy");
        }

        #[cfg(not(feature = "enable-cfg"))]
        {
            let c = Config::default();
            let config = c.0;
            assert_eq!(config.ip, "192.168.1.1");
            assert_eq!(config.port, Some(4433));
            assert_eq!(config.keys.github, "00000000000000000");
            assert_eq!(config.keys.travis.as_ref().unwrap(), "11111111111111111");
        }
    }
}
