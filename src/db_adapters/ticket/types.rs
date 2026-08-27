use entities::user_relations_userrelation::UserRelationId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct CreateWishParams {
    pub use_description: String,
    pub ticket_id: i64,
    pub user_relation_id: UserRelationId,
}
