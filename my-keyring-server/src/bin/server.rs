fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "my_keyring_server,actix_web=debug");
    std::env::set_var("RUST_LOG_STYLE", "always");

    env_logger::Builder::from_default_env()
        .default_format()
        .format_timestamp_micros()
        .try_init()
        .expect("failed to init logger");

    actix_web::rt::System::new().block_on(async { my_keyring_server::main("127.0.0.1:3000").await })
}
