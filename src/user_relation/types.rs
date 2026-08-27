use chrono::NaiveDate;
use diary::DiaryVisible;
use entities::user_relations_userrelation::UserRelationId;
use serde::{Deserialize, Serialize};
use ticket::TicketVisible;

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct ListUserRelationsResponse {
    pub user_relations: Vec<UserRelationVisible>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct UserRelationVisible {
    pub id: UserRelationId,
    pub related_username: String,
    pub giving_ticket_img: Option<String>,
    pub receiving_ticket_img: Option<String>,
    pub use_slack: bool,
    pub first_giving_ticket_date: Option<NaiveDate>,
    pub first_receiving_ticket_date: Option<NaiveDate>,
    pub first_diary_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct SpecialTicketAvailabilityQueryParam {
    pub year: i32,
    pub month: u32,
}

impl SpecialTicketAvailabilityQueryParam {
    pub fn validate(&self) -> Result<(), String> {
        self.validate_year()?;
        self.validate_month()?;
        Ok(())
    }

    fn validate_year(&self) -> Result<(), String> {
        if self.year > 2200 {
            return Err("Year must not be over 2200".to_string());
        }
        if self.year < 2000 {
            return Err("Year must not be under 2000".to_string());
        }
        Ok(())
    }

    fn validate_month(&self) -> Result<(), String> {
        if self.month < 1 || self.month > 12 {
            return Err("Month must be within 1 to 12".to_string());
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchRequest {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub giving_tickets: Vec<TicketVisible>,
    pub receiving_tickets: Vec<TicketVisible>,
    pub diaries: Vec<DiaryVisible>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AvailableTicketsResponse {
    pub oldest: AvailableTicketsOldest,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AvailableTicketsOldest {
    pub normal: Option<TicketVisible>,
    pub special: Option<TicketVisible>,
}
