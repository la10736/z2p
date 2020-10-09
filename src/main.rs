
use z2p::run;

#[cfg(not(tarpaulin_include))]
#[async_std::main]
async fn main() -> tide::Result<()> {
    run().await
}
