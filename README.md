# Configuration Boostrapping Playground

The motivation of this experiment is give a configuration crate the ability to bake in configuration
at compile time, but not have that configuration included in the committed code.

Some extra things that I thought were nice:

- [x] `serde` parsing of the configuration file to a custom `Config` struct type
- [x] configuration file in toml format
- [x] Relative paths for the config are adjusted to be relative to the workspace root at compile time.
  - absolute paths are left alone

## Usage

```sh
# Compile and run tests attempting to use the default path to the configuration file:
# `$CARGO_WORKSPACE_DIR/`
cargo test -p bootstraps2 -- --nocapture

# Provide an absolute path to an alternate configuration file by environment variable.
NYMVPN_CONFIG_PATH=/tmp/nymvpn/nymvpn-config-alt.toml cargo test -p bootstraps2 -- --nocapture

# Provide a relative path to an alternate configuration file by environment variable.
# Relative paths are taken relative to the _WORKSPACE ROOT_
NYMVPN_CONFIG_PATH=nymvpn-config-alt.toml cargo test -p bootstraps2 -- --nocapture

# Compile and run tests WITHOUT using the bootstrap config file. Build using a static constant
# configuration hard-coded in the source.
cargo test -p bootstraps --no-default-features -- --nocapture
```

### Paths relative to the Workspace

By default, while compiling rust wants to use paths relative to crate manifest paths --
i.e. using the provided `CARGO_MANIFEST_DIR`. This is slightly annoying when we want to
use paths relative to the workspace root which cargo does not provide.

In order to do just this (get paths relative to the workspace root) we create the environment
variable `CARGO_WORKSPACE_DIR` with our desired path.

```toml
# .cargo/config.toml

[env]
CARGO_WORKSPACE_DIR = { value = "", relative = true }
```

## Design choice drawbacks / tradeoffs

The example toml config file (`nymvpn-config.toml`) contains the following:

```toml
ip = "127.0.0.1"

[keys]
github = "xxxxxxxxxxxxxxxxx"
travis = "yyyyyyyyyyyyyyyyy"
```

### Version 2

The second iteration uses the [`uneval`](https://docs.rs/uneval/latest/uneval/) crate to take our toml config file and
create an actual instance of the object. This is done in `build.rs` by parsing the toml file using `serde` to a custom
struct defined in `config-types`

Note: config types is required to be a separate crate because it is required by `build.rs` -- any struct in the current crate will not be compiled yet.

- [x] Disable / Enable `build.rs` bootstrap config parsing with a feature
  - [x] when disabled (i.e. the rust crate feature is left off), if a config is provided it is ignored in favor of the in-source default.
  - [x] when enabled, if no config is provided it causes a compile time error

The generated code for this method looks like:

```rs
impl Default for Config {
    fn default() -> Self {
        use config_types::Keys;

        Self(
            BaseConfig {ip: "127.0.0.1".into(),port: None,keys: Keys {github: "xxxxxxxxxxxxxxxxx".into(),travis: Some("yyyyyyyyyyyyyyyyy".into())}}
        )
    }
}
```

When the `config-user` binary is built with only this strategy and looked at for a critical config info using `strings`:

```txt
$ strings debug/config-user | grep "127\.0\.0\.1" -C 5
    n'C
    n'C
    n'C
    l-A
    Wgu
127.0.0.1/home/jmwample/svc/jmwample/rs-bootstraps/target/debug/build/bootstraps2-064e47eac25b3893/out/default.rsxxxxxxxxxxxxxxxxxyyyyyyyyyyyyyyyyy
.gnu_debugaltlin
connection reset
assertion `left ) when slicing `entity not foundk
host unreachable.debug_types.dwo{invalid syntax}
invalid filenamerange end index
```


### Version 1

The first attempt relied on `build.rs` to serialize a toml file **WHOLE** as a const string into a source at
compile time. That `&str` config is then parsed in to a config using things like the `Default` trait.

- One main drawback of this is that the configuration exists in the binary as one string. This means that anyone who
wants to extract the config just needs to run `strings` on the binary.

  - This specifically makes me want a better solution for ingesting and breaking up the string that we are adding at
    compile time. The whole point is to make it more difficult to get an intact version of this config from production
    client builds. Obviously anyone could run a debugger and reconstruct this through careful analysis, but the goal is
    to make that non-trivial (i.e. simply running `strings <path/to/bin>` is too easy).

  - I think the standard I would like to meet is that you have to either manually debug the binary or write a fully
    fledged custom tool to extract the bootstrapped config.

  - This construction makes it unnecessary to use a separate `config-types` crate as we don't actually need the `Config`
    object to be available to the `bootstraps` crate at the stage where `build.rs` runs.

- [x] Disable / Enable / Require `build.rs` bootstrap config parsing with a feature
  - [x] when disabled, if a config is provided it is ignored in favor of the in-source default.
  - [x] when enabled, if no config is provided a warning is printed during compilation and the in-source default is used
  - [x] when required, if no config is provided compilation fails

The generated code for this method looks like:

```rs
/// Generated Value for configuration hardcoded at compile time
pub(crate) const BOOTSTRAP_CONFIG_STR: &str = r#"
ip = "127.0.0.1"

[keys]
github = "xxxxxxxxxxxxxxxxx"
travis = "yyyyyyyyyyyyyyyyy""#;
```

When the `config-user` binary is built with only this strategy and looked at for a critical config info using `strings` we can just plainly see the whole config in toml format.

```txt
$ strings debug/config-user | grep "127\.0\.0\.1" -C 5
    n'C
    l-A
    n'C
    Wgu
gdb_load_rust_pretty_printers.py
ip = "127.0.0.1"
[keys]
github = "xxxxxxxxxxxxxxxxx"
travis = "yyyyyyyyyyyyyyyyy"
ip = "192.168.1.1"
port = 4433
```
