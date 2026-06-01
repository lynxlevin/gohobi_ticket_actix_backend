mod encryptor;

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DbBackend, DbConn, DbErr};

use crate::settings::types::Settings;
pub use encryptor::{decode_and_decrypt, encrypt_and_encode};

pub async fn init_db(settings: &Settings) -> Result<DbConn, DbErr> {
    let database_url = &settings.database.url;
    let db = Database::connect(database_url)
        .await
        .expect("Failed to open DB connection.");
    let db_conn = match db.get_database_backend() {
        DbBackend::MySql => {
            let url = format!("{}", &database_url);
            Database::connect(&url).await.expect("Failed to open DB connection.")
        }
        DbBackend::Postgres => {
            let url = format!("{}", &database_url);
            Database::connect(&url).await.expect("Failed to open DB connection.")
        }
        DbBackend::Sqlite => db,
    };
    Migrator::up(&db_conn, None).await.unwrap();
    Ok(db_conn)
}
