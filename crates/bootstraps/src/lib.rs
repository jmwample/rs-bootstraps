use config_types::BaseConfig;

pub struct Config(BaseConfig);

include!(concat!(env!("OUT_DIR"), "/default.rs"));

impl AsRef<BaseConfig> for Config {
    fn as_ref(&self) -> &BaseConfig {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        #[cfg(feature = "enable-cfg")]
        {
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
