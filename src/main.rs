use z2p::run;

#[cfg(not(tarpaulin_include))]
#[async_std::main]
async fn main() -> tide::Result<()> {
    run("127.0.0.1:8080").await.map_err(|e| e.into())
}
