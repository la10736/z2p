use rstest::rstest;

pub mod utils;

use utils::{app, App};

#[rstest]
async fn health_check_works(app: App) {
    let response = surf::get(format!("http://{}/health_check", app.address))
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.len());
}
