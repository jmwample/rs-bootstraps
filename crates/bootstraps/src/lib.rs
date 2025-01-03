use std::str::FromStr;

#[cfg(feature = "pre-loaded")]
include!(concat!(env!("OUT_DIR"), "/obfuscated.rs"));

pub struct Config(config_types::Config);

const DEFAULT_CONFIG_STR: &str = r#"ip = "127.0.0.1"

[keys]
github = "xxxxxxxxxxxxxxxxx"
travis = "yyyyyyyyyyyyyyyyy"
"#;

impl Default for Config {
    fn default() -> Self {
        Self(
            config_types::Config::from_str(DEFAULT_CONFIG_STR)
                .expect("failed to parse default config"),
        )
    }
}

pub fn add(left: f64, right: f64) -> f64 {
    left + right + ALPHAS.iter().sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("{:?}", ALPHAS);
        println!("{}", add(1.0, 2.0))
    }
}
