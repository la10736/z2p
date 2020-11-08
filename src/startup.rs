use mongodb::Database;

use crate::{configuration::DatabaseSettings, handlers::*};

#[derive(Clone)]
pub struct State {
    db: Database,
}

impl State {
    pub async fn new(cfg: &DatabaseSettings) -> tide::Result<Self> {
        let mongo = mongodb::Client::with_uri_str(&cfg.connection_string()).await?;
        Ok(Self {
            db: mongo.database(&cfg.name),
        })
    }

    pub fn db(&self) -> &Database {
        &self.db
    }
}

pub async fn run(cfg: DatabaseSettings) -> tide::Server<State> {
    let state = State::new(&cfg).await.unwrap();
    let mut app = tide::with_state(state);
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscriptions);
    app
}
