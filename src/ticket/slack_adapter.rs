use std::sync::Arc;

use common::settings::types::Settings;
use db_adapters::user_relation::types::UserRelationWithName;
use entities::tickets_ticket;
use serde_json::json;

pub fn get_message(
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

pub async fn send_slack_message(
    message: &serde_json::Value,
    settings: &Settings,
) -> Result<(), String> {
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
        .map_err(|e| e.to_string())?;
    Ok(())
}
