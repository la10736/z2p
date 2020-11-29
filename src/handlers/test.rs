use crate::{
    configuration::DatabaseSettings,
    state::{State, StateTrait},
};

pub(crate) struct AppBuilder<S: StateTrait>(tide::Server<S>);

impl<S: 'static + StateTrait> AppBuilder<S> {
    pub(crate) fn take(self) -> tide::Server<S> {
        self.0
    }

    pub(crate) fn post(mut self, post: impl tide::Endpoint<S>) -> Self {
        self.0.at("/").post(post);
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(mut self, get: impl tide::Endpoint<S>) -> Self {
        self.0.at("/").get(get);
        self
    }
}

impl AppBuilder<State> {
    pub async fn from_dbcfg(cfg: &DatabaseSettings) -> Self {
        State::new(&cfg).await.unwrap().into()
    }
}

impl From<State> for AppBuilder<State> {
    fn from(state: State) -> Self {
        Self(tide::with_state(state))
    }
}

pub(crate) fn fake_db_settings() -> DatabaseSettings {
    DatabaseSettings {
        username: "mongo".to_string(),
        password: "mongo".to_string(),
        port: 12345,
        host: "localhost".to_string(),
        name: "no_name".to_string(),
        connection_timeout: Some(std::time::Duration::from_millis(10)),
    }
}
