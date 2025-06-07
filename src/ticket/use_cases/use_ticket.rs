use std::sync::Arc;

use chrono::Utc;
use common::{errors::use_case_errors::UseCaseError, settings::types::Settings};
use db_adapters::{
    ticket::{types::UpdateTicketParams, TicketMutation, TicketQuery},
    user_relation::{types::UserRelationWithName, UserRelationQuery},
};
use entities::{tickets_ticket, users_user};
use serde_json::json;

use crate::{TicketVisible, UseTicketParams};

pub async fn use_ticket(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    ticket_id: i64,
    params: UseTicketParams,
    settings: &Settings,
) -> Result<TicketVisible, UseCaseError> {
    let ticket = ticket_query
        .filter_which_user_has_access(user.id)
        .exclude_draft_tickets()
        .get_by_id(ticket_id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if ticket.giving_user_id == user.id {
        return Err(UseCaseError::Forbidden);
    };

    let user_relation = user_relation_query
        .find_by_id_with_user_name(ticket.user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if user_relation.use_slack {
        let message = get_message(&ticket, &user_relation, &params.use_description);
        send_slack_message(&message, &settings).await?;
    }

    ticket_mutation
        .update(
            ticket,
            UpdateTicketParams {
                use_description: Some(params.use_description),
                use_date: Some(Utc::now().date_naive()),
                ..Default::default()
            },
        )
        .await
        .map(|ticket| TicketVisible::from(ticket))
        .map_err(|_| UseCaseError::InternalServerError)
}

// MYMEMO: add slack_adaptor
fn get_message(
    ticket: &tickets_ticket::Model,
    user_relation: &UserRelationWithName,
    use_description: &str,
) -> serde_json::Value {
    let (giving_user_name, receiving_user_name) =
        match user_relation.user_1_id == ticket.giving_user_id {
            true => (&user_relation.user_1_name, &user_relation.user_2_name),
            false => (&user_relation.user_2_name, &user_relation.user_1_name),
        };
    match ticket.is_special {
        true => {
            json!({
                    "text": format!("{}が特別チケットを使ったよ", receiving_user_name),
                    "blocks": [
                {
                    "type": "section",
                    "text": {"type": "mrkdwn", "text": ":star: :star: :star: 特別チケット :star: :star: :star:"},
                },
                {"type": "section", "text": {"type": "mrkdwn", "text": format!("{}へ:\n{}", giving_user_name, use_description)}},
                {
                    "type": "section",
                    "text": {"type": "mrkdwn", "text": format!("使ったチケット: \n```\n{}\n```", ticket.description)},
                },
                    ],
                }
            )
        }
        false => {
            json!({
                    "text": format!("{}がチケットを使ったよ", receiving_user_name),
                    "blocks": [
                        {"type": "section", "text": {"type": "mrkdwn", "text": format!("{}へ:\n{}", giving_user_name, use_description)}},
                        {
                            "type": "section",
                            "text": {"type": "mrkdwn", "text": format!("使ったチケット: \n```\n{}\n```", ticket.description)},
                        },
                    ],
                }
            )
        }
    }
}

async fn send_slack_message(
    message: &serde_json::Value,
    settings: &Settings,
) -> Result<(), UseCaseError> {
    let config = actix_tls::connect::rustls_0_23::reexports::ClientConfig::builder()
        .with_root_certificates(actix_tls::connect::rustls_0_23::webpki_roots_cert_store())
        .with_no_client_auth();
    let client = awc::Client::builder()
        .connector(awc::Connector::new().rustls_0_23(Arc::new(config)))
        .finish();
    let url = format!(
        "{}{}",
        settings.application.slack_host, settings.application.slack_incoming_webhook_path
    );
    let _res = client
        .post(url)
        .content_type("application/json")
        .send_json(message)
        .await
        .map_err(|e| {
            dbg!(e);
            UseCaseError::InternalServerError
        })?;
    Ok(())
}
