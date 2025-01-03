
fn main() {
    #[cfg(feature = "pre-loaded")]
    preload::load_hardcoded_hostfile();

    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(feature = "pre-loaded")]
mod preload {
    const PREAMBLE: &str = r#"
/// Generated Value for configuration hardcoded at compile time
pub(crate) const ALPHAS:[f64; "#;

    pub(crate) fn load_hardcoded_hostfile() {
        // allow the name of the file we draw hardcoded values from to be set by an
        // environment variable at compile time.
        let cfg_file_name = option_env!("NYM_HOSTFILE_CONFIG").unwrap_or("data.txt");

        // set reasons to rebuild
        println!("cargo:rerun-if-changed={cfg_file_name}");
        println!("cargo:rerun-if-env-changed=NYM_HOSTFILE_CONFIG");

        let lines: Vec<String> = match std::fs::read_to_string(cfg_file_name) {
            // split the string into an iterator of string slices
            Ok(r) => r.lines().map(String::from).collect(),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Vec::new(),
                _ => panic!("{e}"),
            },
        };

        // creating a string with the values from our config file and the proper length
        let mut array_string = String::from(PREAMBLE);
        array_string.push_str(lines.len().to_string().as_str());
        array_string.push_str("] = [\r\n");

        for line in &lines {
            // a little bit of formatting is happening as well
            array_string.push_str("\u{20}\u{20}\u{20}\u{20}");
            array_string.push_str(line);
            array_string.push_str(",\r\n");
        }
        array_string.push_str("];\r\n");

        // write the string to a file. OUT_DIR environment variable is defined by cargo
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let dest_path = std::path::Path::new(&out_dir).join("obfuscated.rs");
        std::fs::write(&dest_path, array_string).unwrap();
    }
}
