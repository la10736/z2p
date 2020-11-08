#[cfg(not(tarpaulin_include))]
#[async_std::main]
async fn main() -> tide::Result<()> {
    let configs = z2p::configuration::get_configuration().expect("Failed to read configuration");

    let host = format!("{}:{}", configs.application_host, configs.application_port);
    z2p::run(configs.database)
        .await
        .listen(host)
        .await
        .map_err(|e| e.into())
}
