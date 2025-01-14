use config_types::BaseConfig;
use std::{
    io::{BufWriter, Write},
    str::FromStr,
};

fn main() {
    write_conditional_default();

    println!("cargo:rerun-if-changed=build.rs");
}

const DEFAULT_CONFIG_PREAMBLE: &[u8] = br#"
impl Default for Config {
    fn default() -> Self {
        use config_types::{Keys, PacketType};

        Self("#;

const DEFAULT_CONFIG_EPILOGUE: &[u8] = br#"
        )
    }
}"#;

#[cfg(feature = "enable-cfg")]
const CUSTOM_BREAK: &[u8] = br#"
            #[cfg(feature="enable-cfg")]
            "#;

#[cfg(not(feature = "enable-cfg"))]
const DEFAULT_BREAK: &[u8] = br#"
            #[cfg(not(feature="enable-cfg"))]
            "#;

fn write_conditional_default() {
    // creating a string with the values from our default Config object
    let mut array_string = BufWriter::new(Vec::new());
    array_string.write(DEFAULT_CONFIG_PREAMBLE).unwrap();

    #[cfg(feature = "enable-cfg")]
    {
        array_string.write(CUSTOM_BREAK).unwrap();
        write_custom_config(&mut array_string);
    }

    #[cfg(not(feature = "enable-cfg"))]
    {
        let default_config = BaseConfig::from_str(config_types::DEFAULT_CONFIG_TOML_STR).unwrap();
        array_string.write(DEFAULT_BREAK).unwrap();
        uneval::write(default_config, &mut array_string).unwrap();
    }

    array_string.write(DEFAULT_CONFIG_EPILOGUE).unwrap();

    // write the string to a file. OUT_DIR environment variable is defined by cargo
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("default.rs");

    let out_str = String::from_utf8(array_string.into_inner().unwrap()).unwrap();
    std::fs::write(&dest_path, out_str).unwrap();
}

#[cfg(feature = "enable-cfg")]
pub(crate) fn write_custom_config(out: impl Write) {
    // allow the name of the file we draw hardcoded values from to be set by an
    // environment variable at compile time.
    let cfg_file_name = option_env!("NYMVPN_CONFIG_PATH").unwrap_or("nymvpn-config.toml");

    // set reasons to rebuild
    println!("cargo:rerun-if-changed={cfg_file_name}");
    println!("cargo:rerun-if-env-changed=NYMVPN_CONFIG_PATH");

    let path = std::path::PathBuf::from(cfg_file_name);
    let cfg_file_path = if path.is_absolute() {
        path
    } else {
        // CARGO_WORKSPACE_DIR is set in .cargo/config.toml - it is NOT provided by cargo itself
        let workspace_path = std::env::var("CARGO_WORKSPACE_DIR").unwrap();
        std::path::Path::new(&workspace_path).join(cfg_file_name)
    };

    let config_str: String = std::fs::read_to_string(cfg_file_path).unwrap();
    let config = BaseConfig::from_str(&config_str).unwrap();

    uneval::write(config, out).unwrap();
}
