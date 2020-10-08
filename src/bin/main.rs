use z2p::hello;

use tide::Request;

#[cfg(not(tarpaulin_include))]
#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/").get(greet);
    app.at("/:name").get(greet);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn greet(req: Request<()>) -> tide::Result {
    let message = req
        .param::<String>("name")
        .map(|name| format!("Hello, {}!", name))
        .unwrap_or(hello().to_owned());
    Ok(message.into())
}
