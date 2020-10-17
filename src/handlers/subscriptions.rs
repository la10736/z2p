use tide::{convert::Deserialize, Request};

#[derive(Deserialize, Debug)]
struct Subscribe {
    name: String,
    email: String,
}

pub(crate) async fn subscriptions(mut req: Request<()>) -> tide::Result {
    Ok(match req.body_form::<Subscribe>().await {
        Ok(_) => tide::Response::new(200),
        Err(_) => tide::Response::new(400),
    })
}
