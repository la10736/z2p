use rstest::rstest;
use std::net::SocketAddr;

pub mod utils;

use utils::app;

mod subscribe {

    use crate::utils::App;

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

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct User {
        pub name: String,
        pub email: String,
    }

    impl From<mongodb::bson::Document> for User {
        fn from(d: mongodb::bson::Document) -> Self {
            User {
                name: d.get_str("name").unwrap().to_owned(),
                email: d.get_str("email").unwrap().to_owned(),
            }
        }
    }

    #[rstest]
    async fn should_accept_a_valid_forma_data(app: App) {
        let response = do_request(
            &app.address,
            "name=De%20Domenico&email=antonio_de_domenico%40gmail.com",
        )
        .await;

        assert_eq!(200, response.status());
        let user = app
            .db
            .collection("subscriptions")
            .find_one(None, None)
            .await
            .expect("Cannot fetch user");

        assert_eq!(
            Some(User {
                name: "De Domenico".to_owned(),
                email: "antonio_de_domenico@gmail.com".to_owned()
            }),
            user.map(|d| d.into())
        );
    }

    #[rstest(
        body,
        case::missed_email("name=De%20Domenico"),
        case::missed_name("email=antonio_de_domenico%40gmail.com"),
        case::missed_both("")
    )]
    async fn should_returns_a_400_when_data_is_missing(app: App, body: &str) {
        let response = do_request(&app.address, body).await;

        assert_eq!(400, u16::from(response.status()));
    }
}
