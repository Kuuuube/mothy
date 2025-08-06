pub struct Database {
    #[expect(dead_code)]
    pool: sqlx::PgPool,
}

impl Database {
    /// Gets an instance of the database.
    ///
    /// # Panics
    ///
    /// Will panic if `DATABASE_URL` is not set properly inside the envirnoment.
    pub async fn init() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set.");

        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect(&database_url)
            .await
            .expect("Failed to connect to database!");

        sqlx::migrate!("../migrations")
            .run(&pool)
            .await
            .expect("Could not run migrations.");

        Self { pool }
    }
}
