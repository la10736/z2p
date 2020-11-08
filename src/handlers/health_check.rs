use tide::Request;

use crate::startup::State;

pub(crate) async fn health_check(_req: Request<State>) -> tide::Result {
    Ok("".into())
}
