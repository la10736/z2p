use tide::listener::ToListener;
use tide::Request;

pub fn run(
    listener: impl ToListener<()>,
) -> impl std::future::Future<Output = std::io::Result<()>> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.at("/subscriptions").post(subscriptions);
    app.listen(listener)
}

pub(crate) async fn health_check(_req: Request<()>) -> tide::Result {
    Ok("".into())
}

use tide::convert::Deserialize;
// use http_types::{Body, Method, Request, Url};

#[derive(Deserialize, Debug)]
struct Subscribe {
    name: String,
    email: String,
}

pub(crate) async fn subscriptions(mut req: Request<()>) -> tide::Result {
    Ok(match req.body_form::<Subscribe>().await {
        Ok(_) => {
            tide::Response::new(200)
        }
        Err(_) => tide::Response::new(400),
    })
}
