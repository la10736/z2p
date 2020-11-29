use mongodb::bson::doc;
use tide::{convert::Deserialize, Request, StatusCode};
use tracing::{error, info};

use crate::{
    repository::{User, UsersRepository},
    state::StateTrait,
};

#[derive(Deserialize, Debug)]
struct Subscribe {
    name: String,
    email: String,
}

impl Into<User> for Subscribe {
    fn into(self) -> User {
        User {
            name: self.name,
            email: self.email,
        }
    }
}

#[tracing::instrument(name = "Adding a new subscriber", skip(req))]
pub(crate) async fn subscriptions<S: StateTrait>(mut req: Request<S>) -> tide::Result {
    let subscriber = req.body_form::<Subscribe>().await.map_err(|mut e| {
        e.set_status(StatusCode::BadRequest);
        e
    })?;
    let subscribe_result = req
        .state()
        .users_repository()
        .create(subscriber.into())
        .await;
    Ok(match subscribe_result {
        Ok(_) => {
            info!("New subcriber saved");
            StatusCode::Ok
        }
        Err(e) => {
            error!("Failed to save suscriber: {:?}", e);
            StatusCode::ServiceUnavailable
        }
    }
    .into())
}

#[cfg(test)]
mod tests {
    use crate::{handlers::test::fake_db_settings, handlers::test::AppBuilder};
    use tide::http::{Method, Request, Response, Url};

    use super::subscriptions;

    #[async_std::test]
    async fn should_return_service_unavailable_if_db_is_down() -> tide::Result<()> {
        let app = AppBuilder::from_dbcfg(&fake_db_settings())
            .await
            .post(subscriptions)
            .take();

        let url = Url::parse("https://example.com").unwrap();
        let mut req = Request::new(Method::Post, url);
        req.set_body("name=De%20Domenico&email=antonio_de_domenico%40gmail.com");
        let res: Response = app.respond(req).await?;

        assert_eq!(tide::StatusCode::ServiceUnavailable, res.status());
        Ok(())
    }
}
