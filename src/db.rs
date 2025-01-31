use diesel::pg::PgConnection;
use diesel::r2d2;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;
use std::error::Error;

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub type DbError = Box<dyn Error + Send + Sync + 'static>;
/// Initializes and returns a connection pool for the PostgreSQL database.
///
/// # Panics
/// This function panics if the `DATABASE_URL` environment variable is missing
/// or if the connection pool cannot be created.
pub fn initialize_db_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("missing DATABASE_URL in .env");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool")
}
/// Runs pending database migrations.
///
/// # Errors
/// Returns an error if the migration process fails.
pub fn run_migrations(
    connection: &mut impl MigrationHarness<diesel::pg::Pg>,
) -> Result<(), DbError> {
    connection.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");
    Ok(())
}
