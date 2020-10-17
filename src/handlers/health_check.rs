use tide::Request;

pub(crate) async fn health_check(_req: Request<()>) -> tide::Result {
    Ok("".into())
}
