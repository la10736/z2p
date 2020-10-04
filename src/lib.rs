pub fn hello() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!("Hello, world!", hello());
    }
}
