fn main() {
    let config = bootstraps::Config::default();
    println!("{}", config.as_ref().ip);

    let config2 = bootstraps2::Config::default();
    println!("{}", config2.as_ref().ip);
}
