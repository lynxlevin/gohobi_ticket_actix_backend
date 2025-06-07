use chrono::Utc;
use entities::user_relations_userrelation;
use sea_orm::Set;

pub fn user_relation(user_1_id: i64, user_2_id: i64) -> user_relations_userrelation::ActiveModel {
    let now = Utc::now();
    user_relations_userrelation::ActiveModel {
        user_1_id: Set(user_1_id),
        user_2_id: Set(user_2_id),
        use_slack: Set(false),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    }
}

pub trait UserRelationFactory {
    fn user_1_giving_ticket_img(
        self,
        img: Option<String>,
    ) -> user_relations_userrelation::ActiveModel;
    fn user_2_giving_ticket_img(
        self,
        img: Option<String>,
    ) -> user_relations_userrelation::ActiveModel;
    fn use_slack(self, use_slack: bool) -> user_relations_userrelation::ActiveModel;
}

impl UserRelationFactory for user_relations_userrelation::ActiveModel {
    fn user_1_giving_ticket_img(
        mut self,
        img: Option<String>,
    ) -> user_relations_userrelation::ActiveModel {
        self.user_1_giving_ticket_img = Set(img);
        self
    }

    fn user_2_giving_ticket_img(
        mut self,
        img: Option<String>,
    ) -> user_relations_userrelation::ActiveModel {
        self.user_2_giving_ticket_img = Set(img);
        self
    }

    fn use_slack(mut self, use_slack: bool) -> user_relations_userrelation::ActiveModel {
        self.use_slack = Set(use_slack);
        self
    }
}
