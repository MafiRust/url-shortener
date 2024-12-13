#[derive(Clone)]
pub struct State {
    database: deadpool_postgres::Pool,
}

impl State {
    #[must_use]
    pub fn new(database: deadpool_postgres::Pool) -> Self {
        Self { database }
    }

    pub async fn database_client(
        &self,
    ) -> Result<deadpool_postgres::Client, deadpool_postgres::PoolError> {
        self.database.get().await
    }
}
