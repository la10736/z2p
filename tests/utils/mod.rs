use std::net::SocketAddr;

use async_std::net::TcpListener;
use rstest::fixture;
use z2p::run;

#[fixture]
pub fn app() -> SocketAddr {
    let listener = async_std::task::block_on(async {
        TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Cannot bind server address")
    });

    let address = listener.local_addr().expect("Cannot get server address");

    async_std::task::spawn(run(listener));

    address
}
