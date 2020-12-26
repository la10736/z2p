use z2p::telemetry::{get_subscriber, init_subscriber};

#[cfg(not(tarpaulin_include))]
#[async_std::main]
async fn main() -> tide::Result<()> {
    init_subscriber(get_subscriber("z2p", "info"));

    let configs = z2p::configuration::get_configuration().expect("Failed to read configuration");
    let host = format!("{}:{}", configs.application.host, configs.application.port);
    z2p::run(configs.database)
        .await
        .listen(host)
        .await
        .map_err(|e| e.into())
}
