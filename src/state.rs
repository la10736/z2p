use crate::{
    adapters::mongodb_repository::MongoUserRepository, configuration::DatabaseSettings, repository,
};

#[derive(Clone)]
pub struct State {
    users_repository: MongoUserRepository,
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
            users_repository: MongoUserRepository::new(mongo.database(&cfg.name)),
        })
    }
}
