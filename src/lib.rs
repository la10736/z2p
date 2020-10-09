use tide::Request;

pub async fn run() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/health_check").get(health_check);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

pub(crate) async fn health_check(_req: Request<()>) -> tide::Result {
    Ok("".into())
}
