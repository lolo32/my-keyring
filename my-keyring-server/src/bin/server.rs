use log::trace;
use saphir::prelude::SaphirError;

fn main() -> Result<(), SaphirError> {
    std::env::set_var("RUST_LOG", "my_keyring_server");
    std::env::set_var("RUST_LOG_STYLE", "always");
    env_logger::Builder::from_default_env()
        .default_format()
        .format_timestamp_micros()
        .try_init()
        .expect("failed to init logger");

    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(my_keyring_server::main("127.0.0.1:3000"))
}
