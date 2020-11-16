use crate::{configuration::DatabaseSettings, handlers::*};

#[derive(Clone)]
pub struct State {
    users_repository: MongoUserRepository,
}

#[derive(Clone)]
pub(crate) struct MongoUserRepository {
    db: Database,
}

pub(crate) mod repository {
    use thiserror::Error;
    pub(crate) type Result<T> = std::result::Result<T, Error>;
    #[derive(Error, Debug)]
    pub(crate) enum Error {
        #[error("Cannot insert entry '{entry_desc}'")]
        InsertDb {
            entry_desc: String,
            #[source]
            source: Box<dyn std::error::Error>,
        },
    }
    #[derive(Debug)]
    pub(crate) struct User {
        pub(crate) name: String,
        pub(crate) email: String,
    }

    #[async_trait::async_trait]
    pub(crate) trait UsersRepository {
        async fn create(&self, user: User) -> Result<()>;
    }
}

use mongodb::{bson::doc, Database};

#[async_trait::async_trait]
impl repository::UsersRepository for MongoUserRepository {
    async fn create(&self, user: repository::User) -> repository::Result<()> {
        let doc = doc! { "name": &user.name, "email": &user.email };
        self.db
            .collection("subscriptions")
            .insert_one(doc, None)
            .await
            .map_err(|e| repository::Error::InsertDb {
                entry_desc: format!("{:?}", &user),
                source: Box::new(e),
            })?;
        Ok(())
    }
}

pub(crate) trait StateTrait: Clone + Send + Sync {
    type UserRepository: repository::UsersRepository;

    fn users_repository(&self) -> &Self::UserRepository;
}

impl StateTrait for State {
    type UserRepository = MongoUserRepository;

    fn users_repository(&self) -> &Self::UserRepository {
        &self.users_repository
    }
}

impl State {
    pub async fn new(cfg: &DatabaseSettings) -> tide::Result<Self> {
        let client_options = mongodb::options::ClientOptions::parse(&cfg.connection_string())
            .await
            .map(|mut opts| {
                opts.server_selection_timeout = cfg.connection_timeout;
                opts.connect_timeout = cfg.connection_timeout;
                opts
            })?;
        let mongo = mongodb::Client::with_options(client_options)?;
        Ok(Self {
            users_repository: MongoUserRepository {
                db: mongo.database(&cfg.name),
            },
        })
    }
}

pub async fn run(cfg: DatabaseSettings) -> tide::Server<State> {
    let state = State::new(&cfg).await.unwrap();
    let mut app = tide::with_state(state);
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscriptions);
    app
}
