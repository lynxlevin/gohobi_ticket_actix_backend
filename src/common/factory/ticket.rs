use std::{collections::HashMap, future::Future};

use chrono::{Days, NaiveDate, Utc};
use entities::{
    custom_types::TicketStatus,
    tickets_ticket::{ActiveModel, Entity, Model},
    user_relations_userrelation::UserRelationId,
    users_user::UserId,
    wish,
};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set};

use crate::{db::Db, factory::wish as wish_factory};

pub fn ticket(giving_user_id: UserId, user_relation_id: UserRelationId) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
        description: Set(String::default()),
        gift_date: Set(now.date_naive()),
        status: Set(TicketStatus::default().to_value()),
        is_special: Set(false),
        giving_user_id: Set(giving_user_id),
        user_relation_id: Set(user_relation_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
}

pub trait TicketFactory {
    fn description(self, description: String) -> ActiveModel;
    fn gift_date(self, gift_date: NaiveDate) -> ActiveModel;
    fn status(self, status: String) -> ActiveModel;
    fn is_special(self, is_special: bool) -> ActiveModel;
    fn insert_with_wish(self, db: &Db) -> impl Future<Output = Result<(Model, wish::Model), DbErr>> + Send;
}

impl TicketFactory for ActiveModel {
    fn description(mut self, description: String) -> ActiveModel {
        self.description = Set(description);
        self
    }

    fn gift_date(mut self, gift_date: NaiveDate) -> ActiveModel {
        self.gift_date = Set(gift_date);
        self
    }

    fn status(mut self, status: String) -> ActiveModel {
        self.status = Set(status);
        self
    }

    fn is_special(mut self, is_special: bool) -> ActiveModel {
        self.is_special = Set(is_special);
        self
    }

    async fn insert_with_wish(self, db: &Db) -> Result<(Model, wish::Model), DbErr> {
        let ticket = self.insert(&db.db).await?;
        let wish = wish_factory(&ticket).insert(&db.db).await?;
        Ok((ticket, wish))
    }
}

#[derive(Default)]
pub struct TicketParam {
    pub name: String,
    pub description: Option<String>,
    pub n_days_ago: i64,
    pub status: TicketStatus,
    pub is_special: bool,
    pub giving_user_id: UserId,
    pub user_relation_id: UserRelationId,
}

pub async fn create_tickets(params: Vec<TicketParam>, db: &Db) -> Result<HashMap<String, Model>, DbErr> {
    let today = Utc::now().date_naive();
    let tickets = params.iter().map(|param| {
        let gift_date = if param.n_days_ago > 0 {
            today
                .checked_sub_days(Days::new(param.n_days_ago.unsigned_abs()))
                .unwrap()
        } else {
            today
                .checked_add_days(Days::new(param.n_days_ago.unsigned_abs()))
                .unwrap()
        };
        let ticket = ticket(param.giving_user_id, param.user_relation_id)
            .gift_date(gift_date)
            .is_special(param.is_special)
            .status(param.status.to_value());
        if param.description.is_some() {
            ticket.description(param.description.clone().unwrap())
        } else {
            ticket.description(param.name.clone())
        }
    });
    let tickets = Entity::insert_many(tickets).exec_with_returning(&db.db).await?;

    Ok(tickets
        .into_iter()
        .zip(params)
        .fold(HashMap::new(), |mut acc, (ticket, param)| {
            acc.entry(param.name.to_string()).or_insert(ticket);
            acc
        }))
}
