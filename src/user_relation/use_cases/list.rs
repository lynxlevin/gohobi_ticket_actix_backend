use crate::{ListUserRelationsResponse, UserRelationVisible};
use common::errors::use_case_errors::UseCaseError;
use db_adapters::user_relation::UserRelationQuery;
use entities::users_user;

pub async fn list_user_relations(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
) -> Result<ListUserRelationsResponse, UseCaseError> {
    let user_relations = match user_relation_query.find_related_by_user_id(user.id).await {
        Ok(user_relations) => user_relations,
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    Ok(ListUserRelationsResponse {
        user_relations: user_relations
            .iter()
            .map(|r| {
                let (related_username, giving_ticket_img, receiving_ticket_img) =
                    match user.id == r.user_1_id {
                        true => (
                            r.user_2_name.clone(),
                            r.user_1_giving_ticket_img.clone(),
                            r.user_2_giving_ticket_img.clone(),
                        ),
                        false => (
                            r.user_1_name.clone(),
                            r.user_2_giving_ticket_img.clone(),
                            r.user_1_giving_ticket_img.clone(),
                        ),
                    };
                UserRelationVisible {
                    id: r.id,
                    related_username,
                    giving_ticket_img,
                    receiving_ticket_img,
                    use_slack: r.use_slack,
                    first_diary_date: r.first_diary_date,
                }
            })
            .collect(),
    })
}
