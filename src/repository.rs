use thiserror::Error;
pub(crate) type Result<T> = std::result::Result<T, Error>;
#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Cannot insert entry '{entry_desc}'")]
    InsertDb {
        entry_desc: String,
        #[source]
        source: Box<dyn std::error::Error>,
    },
}
#[derive(Debug)]
pub(crate) struct User {
    pub(crate) name: String,
    pub(crate) email: String,
}

#[async_trait::async_trait]
pub(crate) trait UsersRepository {
    async fn create(&self, user: User) -> Result<()>;
}
