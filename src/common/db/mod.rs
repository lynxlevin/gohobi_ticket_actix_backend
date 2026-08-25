mod encryptor;

use sea_orm::{Database, DbConn, DbErr};

use crate::settings::types::Settings;
pub use encryptor::{decode_and_decrypt, encrypt_and_encode};

pub async fn init_db(settings: &Settings) -> Result<DbConn, DbErr> {
    let database_url = &settings.database.url;
    let db = Database::connect(database_url)
        .await
        .expect("Failed to open DB connection.");

    db.get_schema_registry("entities::*")
        .sync(&db)
        .await
        .expect("Failed in DB migration.");

    Ok(db)
}
