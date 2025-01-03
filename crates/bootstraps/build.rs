fn main() {
    #[cfg(feature = "pre-loaded")]
    preload::load_hostfile();

    println!("cargo:rerun-if-changed=build.rs");
}

macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

#[cfg(feature = "pre-loaded")]
mod preload {
    // use config_types::Config;
    // use std::str::FromStr;

    const PREAMBLE: &str = r#"
    /// Generated Value for configuration hardcoded at compile time
    pub(crate) const BOOTSTRAP_CONFIG_STR: &str = r#""#;

    pub(crate) fn load_hostfile() {
        // allow the name of the file we draw hardcoded values from to be set by an
        // environment variable at compile time.
        let cfg_file_name = option_env!("NYMVPN_HOSTFILE_CONFIG").unwrap_or("nymvpn-config.toml");

        // set reasons to rebuild
        println!("cargo:rerun-if-changed={cfg_file_name}");
        println!("cargo:rerun-if-env-changed=NYMVPN_HOSTFILE_CONFIG");

        let path = std::path::PathBuf::from(cfg_file_name);
        let cfg_file_path = if path.is_absolute() {
            path
        } else {
            // CARGO_WORKSPACE_DIR is set in .cargo/config.toml - it is NOT provided by cargo itself
            let workspace_path = std::env::var("CARGO_WORKSPACE_DIR").unwrap();
            std::path::Path::new(&workspace_path).join(cfg_file_name)
        };

        let config_str: String = match std::fs::read_to_string(cfg_file_path) {
            // split the string into an iterator of string slices
            Ok(r) => r,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    warn!("config bootstrapping was enabled, but no config file was found");
                    String::new()
                }
                _ => panic!("{e}"),
            },
        };

        // let config = Config::from_str(&config_str).unwrap();

        // // creating a string with the values from our config file and the proper length
        let mut array_string = String::from(PREAMBLE);
        array_string.push_str(&config_str);
        array_string.push_str("\"#;\r\n");

        // write the string to a file. OUT_DIR environment variable is defined by cargo
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let dest_path = std::path::Path::new(&out_dir).join("obfuscated.rs");
        std::fs::write(&dest_path, array_string).unwrap();
    }
}
