use std::net::SocketAddr;

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(my_keyring_server::main(&SocketAddr::from((
            [127, 0, 0, 1],
            3000,
        ))))
}
