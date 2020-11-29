use crate::{configuration::DatabaseSettings, handlers::*, state::State};

pub async fn run(cfg: DatabaseSettings) -> tide::Server<State> {
    let state = State::new(&cfg).await.unwrap();
    let mut app = tide::with_state(state);
    app.with(tide_tracing::TraceMiddleware::new());
    app.with(crate::middleware::TraceUuidMiddleware::new());
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscriptions);
    app
}
