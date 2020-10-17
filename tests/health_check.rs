use rstest::rstest;
use std::net::SocketAddr;

mod utils;

use utils::app;

#[rstest]
async fn health_check_works(app: SocketAddr) {
    let response = surf::get(format!("http://{}/health_check", app))
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.len());
}
