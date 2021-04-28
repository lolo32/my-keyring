use log::trace;
use saphir::prelude::SaphirError;

fn main() -> Result<(), SaphirError> {
    std::env::set_var("RUST_LOG", "my_keyring_server");
    env_logger::init();

    trace!("bbb");

    tokio::runtime::Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .block_on(my_keyring_server::main("127.0.0.1:3000"))
}
