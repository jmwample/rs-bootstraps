# Configuration Boostrapping Playground

The motivation of this experiment is give a configuration crate the ability to bake in configuration
at compile time, but not have that configuration included in the committed code. 

Some extra things that I thought were nice:
- [x] `serde` parsing of the configuration file to a custom `Config` type
- [x] configuration file in toml format
- [x] Relative paths for the config are adjusted to be relative to the workspace root at compile time.
    - absolute paths are left as is
- [x] Disable / Enable / Require `build.rs` bootstrap config parsing with a feature
	- [ ] when disabled, if a config is provided it is ignored in favor of the in-source default.
	- [x] when enabled, if no config is provided a warning is printed during compilation and the in-source default is used
	- [ ] when required, if no config is provided compilation fails

## Usage


```sh
# Compile and run tests attempting to use the default path to the configuration file:
# `$CARGO_WORKSPACE_DIR/`
cargo test -p bootstraps -- --nocapture

# Provide an absolute path to an alternate configuration file by environment variable.
NYMVPN_HOSTFILE_CONFIG=/tmp/nymvpn/nymvpn-config-alt.toml cargo test -p bootstraps -- --nocapture

# Provide a relative path to an alternate configuration file by environment variable.
# Relative paths are taken relative to the _WORKSPACE ROOT_
NYMVPN_HOSTFILE_CONFIG=nymvpn-config-alt.toml cargo test -p bootstraps -- --nocapture

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

The way this is currently set up `build.rs` takes a toml file and adds it **WHOLE** as a const string in the source at
compile time. That `&str` config is then parsed in to a config using things like the `Default` trait.

- One main drawback of this is that the configuration exists in the binary as one string. This means that anyone who
wants to extract the config just needs to run strings on the binary.

	- This specifically makes me want a better
	solution for ingesting and breaking up the string that we are adding at compile time. The whole point is to make it
	more difficult to get an intact version of this config from production client builds. Obviously anyone could run a
	debugger and reconstruct this through careful analysis, but the goal is to make that non-trivial (i.e. simply running
	`strings <path/to/bin>` is too easy).

	- I think the standard I would like to meet is that you have to either manually debug the binary or write a fully
	fledged custom tool to extract the bootstrapped config.

	- This construction makes it unnecessary to use a separate `config-types` crate as we don't actually need the
	`Config` object to be available to the `bootstraps` crate at the stage where `build.rs` runs.

- we could instead just have a rust file with a const struct where the rust file itself is the bootstrap configuration

	- this has the significant drawback that a rust file added using the `include!()` macro could just add arbitrary
	functionality rather than just configuration. Which seems like a can of worms I don't want to open.

- we could go through the painstaking process of building up a const struct using calls to `println!()` in `build.rs`,
but this would need constant attention and fixing as the Config struct changes.

- we could (maybe??) use something like [`uneval`](https://docs.rs/uneval/latest/uneval/)