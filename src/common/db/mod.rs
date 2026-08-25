mod encryptor;

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DbConn, DbErr};

use crate::settings::types::Settings;
pub use encryptor::{decode_and_decrypt, encrypt_and_encode};

pub async fn init_db(settings: &Settings) -> Result<DbConn, DbErr> {
    let database_url = &settings.database.url;
    let db = Database::connect(database_url)
        .await
        .expect("Failed to open DB connection.");
    Migrator::up(&db, None).await.unwrap();
    Ok(db)
}
