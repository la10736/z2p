use mongodb::{bson::doc, results::InsertOneResult, Database};
use tide::{convert::Deserialize, Request};

use crate::startup::State;

#[derive(Deserialize, Debug)]
struct Subscribe {
    name: String,
    email: String,
}

async fn insert_subscription(
    db: Database,
    subscribe: Subscribe,
) -> mongodb::error::Result<InsertOneResult> {
    let doc = doc! { "name": subscribe.name, "email": subscribe.email };
    db.collection("subscriptions").insert_one(doc, None).await
}

pub(crate) async fn subscriptions(mut req: Request<State>) -> tide::Result {
    Ok(match req.body_form::<Subscribe>().await {
        Ok(subscribe) => {
            insert_subscription(req.state().db().clone(), subscribe)
                .await
                .unwrap();
            tide::Response::new(200)
        }
        Err(_) => tide::Response::new(400),
    })
}
