use rstest::rstest;
use std::net::SocketAddr;

mod utils;

use utils::app;

mod subscribe {
    use std::time::Duration;

    use super::*;
    use chrono::{DateTime, Utc};
    use mongodb::{bson::doc, options::ClientOptions, Client, Database};
    use surf::Response;
    use z2p::configuration::DatabaseSettings;

    async fn do_request(app: &SocketAddr, body: &str) -> Response {
        surf::post(format!("http://{}/subscriptions", app))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    fn now() -> DateTime<Utc> {
        std::time::SystemTime::now().into()
    }

    fn testname() -> String {
        std::thread::current().name().unwrap().to_string()
    }

    fn sanitize_db_name(name: impl AsRef<str>) -> String {
        name.as_ref().replace(
            &['/', '\\', '.', '“', '*', '<', '>', ':', '|', '?', '$'][..],
            "_",
        )
    }

    fn configurations() -> DatabaseSettings {
        let mut configurations =
            z2p::configuration::get_configuration().expect("Failed to read configurations");
        configurations.database.name = sanitize_db_name(testname());
        configurations.database
    }

    async fn mongodb_client_options(url: &str) -> ClientOptions {
        let mut client_options = ClientOptions::parse(&url)
            .await
            .expect("Cannot parse db connection string");
        client_options.server_selection_timeout = Some(Duration::from_millis(500));
        client_options.connect_timeout = Some(Duration::from_millis(500));

        client_options
    }

    async fn create_db(cfg: &DatabaseSettings) {
        let url = format!(
            "mongodb://{}:{}@{}:{}",
            cfg.username, cfg.password, cfg.host, cfg.port
        );
        let mut client_options = mongodb_client_options(&url).await;
        client_options.app_name = Some("CreateDb".to_string());

        let client = Client::with_options(client_options).expect("Cannot create db client");
        let db = client.database(&cfg.name);

        let collection = db.collection("test_entry__");

        let docs = vec![doc! { "created": now() }];

        // Insert some documents into the "mydb.books" collection.
        collection
            .insert_many(docs, None)
            .await
            .expect("Cannot write new db");
    }

    async fn db(cfg: &DatabaseSettings) -> Database {
        let mut client_options = mongodb_client_options(&cfg.connection_string()).await;
        client_options.app_name = Some(testname());

        Client::with_options(client_options)
            .expect("Cannot create db client")
            .database(&cfg.name)
    }

    #[rstest]
    #[ignore]
    async fn should_accept_a_valid_forma_data(app: SocketAddr) {
        let cfg = configurations();
        create_db(&cfg).await;
        let db = db(&cfg).await;

        let response = do_request(
            &app,
            "name=De%20Domenico&email=antonio_de_domenico%40gmail.com",
        )
        .await;

        assert_eq!(200, response.status());
        let user = db
            .collection("subscriptions")
            .find_one(None, None)
            .await
            .expect("Cannot fetch user");
        assert_eq!(
            Some(doc! {"email": "antonio_de_domenico@gmail.com", "name": "De Domenico"}),
            user
        );
    }

    #[rstest(
        body,
        case::missed_email("name=De%20Domenico"),
        case::missed_name("email=antonio_de_domenico%40gmail.com"),
        case::missed_both("")
    )]
    async fn should_returns_a_400_when_data_is_missing(app: SocketAddr, body: &str) {
        let response = do_request(&app, body).await;

        assert_eq!(400, u16::from(response.status()));
    }
}
