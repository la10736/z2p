use rstest::rstest;
use std::net::SocketAddr;

mod utils;

use utils::app;

mod subscribe {
    use super::*;
    use surf::Response;

    async fn do_request(app: &SocketAddr, body: &str) -> Response {
        surf::post(format!("http://{}/subscriptions", app))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    #[rstest]
    async fn should_accept_a_valid_forma_data(app: SocketAddr) {
        let response = do_request(&app, "name=le%20guin&email=ursula_le_guin%40gmail.com").await;

        assert_eq!(200, response.status())
    }

    #[rstest(
        body,
        case::missed_email("name=le%20guin"),
        case::missed_name("email=ursula_le_guin%40gmail.com"),
        case::missed_both("")
    )]
    async fn should_returns_a_400_when_data_is_missing(app: SocketAddr, body: &str) {
        let response = do_request(&app, body).await;

        assert_eq!(400, u16::from(response.status()));
    }
}
