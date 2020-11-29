#[derive(Clone)]
pub(crate) struct MongoUserRepository {
    db: Database,
}

impl MongoUserRepository {
    pub(crate) fn new(db: Database) -> Self {
        Self { db }
    }
}

use mongodb::{bson::doc, Database};

use crate::repository;

#[async_trait::async_trait]
impl repository::UsersRepository for MongoUserRepository {
    #[tracing::instrument(
        name = "Saving a new subscriber",
        skip(self, user),
        fields(
            name = %user.name,
            email = %user.email,
        )
    )]
    async fn create(&self, user: repository::User) -> repository::Result<()> {
        let doc = doc! { "name": &user.name, "email": &user.email };
        self.db
            .collection("subscriptions")
            .insert_one(doc, None)
            .await
            .map_err(|e| repository::Error::InsertDb {
                entry_desc: format!("{:?}", &user),
                source: Box::new(e),
            })?;
        Ok(())
    }
}
