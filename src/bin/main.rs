use z2p::hello;

#[cfg(not(tarpaulin_include))]
fn main() {
    println!("{}", hello());
}
