#[derive(Debug, Default, Clone)]
pub struct TraceUuidMiddleware;

impl TraceUuidMiddleware {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for TraceUuidMiddleware {
    #[tracing::instrument(
        name = "Mark",
        skip(req, next),
        fields(
            request_id = %uuid::Uuid::new_v4(),
        )
    )]
    async fn handle(&self, req: tide::Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        Ok(next.run(req).await)
    }
}
