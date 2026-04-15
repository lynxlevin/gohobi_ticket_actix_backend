pub use sea_orm_migration::prelude::{async_trait, MigrationTrait, MigratorTrait};

mod m20250507_000001_create_users_table;
mod m20250507_000002_create_user_relations_table;
mod m20250507_000003_create_tickets_table;
mod m20250507_000004_create_diaries_table;
mod m20250507_000005_create_diary_tags_table;
mod m20260301_000001_create_web_push_subscriptions_table;
mod m20260330_000001_create_wishes_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250507_000001_create_users_table::Migration),
            Box::new(m20250507_000002_create_user_relations_table::Migration),
            Box::new(m20250507_000003_create_tickets_table::Migration),
            Box::new(m20250507_000004_create_diaries_table::Migration),
            Box::new(m20250507_000005_create_diary_tags_table::Migration),
            Box::new(m20260301_000001_create_web_push_subscriptions_table::Migration),
            Box::new(m20260330_000001_create_wishes_table::Migration),
        ]
    }
}
