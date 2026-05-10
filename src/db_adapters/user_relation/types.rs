use chrono::NaiveDate;
use sea_orm::FromQueryResult;

#[derive(FromQueryResult)]
pub struct UserRelationWithName {
    pub id: i64,
    pub user_1_giving_ticket_img: Option<String>,
    pub user_2_giving_ticket_img: Option<String>,
    pub user_1_id: i64,
    pub user_2_id: i64,
    pub user_1_name: String,
    pub user_2_name: String,
    pub use_slack: bool,
    pub first_user_1_giving_ticket_date: Option<NaiveDate>,
    pub first_user_2_giving_ticket_date: Option<NaiveDate>,
    pub first_diary_date: Option<NaiveDate>,
}
