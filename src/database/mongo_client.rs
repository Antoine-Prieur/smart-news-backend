use log::info;
use mongodb::{Client, Database};

#[derive(Clone)]
pub struct DatabaseClient {
    pub database: Database,
}

impl DatabaseClient {
    pub async fn new(
        connection_string: &str,
        database_name: &str,
    ) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(connection_string).await?;

        let database = client.database(database_name);

        client
            .database("admin")
            .run_command(mongodb::bson::doc! {"ping": 1})
            .await?;

        info!(
            "Successfully connected to MongoDB database: {}",
            database_name
        );

        Ok(Self { database })
    }

    pub fn get_database(&self) -> Database {
        self.database.clone()
    }
}
