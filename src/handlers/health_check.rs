use tide::Request;

use crate::state::State;

pub(crate) async fn health_check(_req: Request<State>) -> tide::Result {
    Ok("".into())
}
