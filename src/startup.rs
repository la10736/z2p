use tide::listener::ToListener;

use crate::handlers::*;

pub fn run(
    listener: impl ToListener<()>,
) -> impl std::future::Future<Output = std::io::Result<()>> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscriptions);
    app.listen(listener)
}
