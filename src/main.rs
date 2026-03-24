mod cli;
mod error;

use clap::Parser;
use cli::{Cli, MsgType};
use error::AppError;
use pulldown_cmark::{html, Options, Parser as MarkdownParser};
use serde_json::json;
use uuid::Uuid;

fn main() {
    let cli = Cli::parse();

    if let Err(err) = send_message(cli) {
        println!("{err}");
    }
}

fn send_message(cli: Cli) -> Result<(), AppError> {
    let server = cli.server.trim_end_matches('/');
    let access_token = std::fs::read_to_string(&cli.access_token_path)?;
    let access_token = access_token.trim();
    let room_id = resolve_room_id(server, &cli.room, access_token)?;
    let room = urlencoding::encode(&room_id);
    let txn_id = Uuid::new_v4().to_string();
    let txn_id = urlencoding::encode(&txn_id);
    let url = format!(
        "{}/_matrix/client/v3/rooms/{}/send/m.room.message/{}",
        server, room, txn_id
    );

    let payload = match cli.msg_type {
        MsgType::Text => json!({
            "msgtype": "m.text",
            "body": cli.message,
        }),
        MsgType::Notice => json!({
            "msgtype": "m.notice",
            "body": cli.message,
        }),
        MsgType::Emote => json!({
            "msgtype": "m.emote",
            "body": cli.message,
        }),
        MsgType::Markdown => {
            let formatted_body = render_markdown(&cli.message);
            json!({
                "msgtype": "m.text",
                "body": cli.message,
                "format": "org.matrix.custom.html",
                "formatted_body": formatted_body,
            })
        }
    };

    let payload = serde_json::to_string(&payload)?;

    let mut response = ureq::put(&url)
        .config()
        .http_status_as_error(false)
        .build()
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .send(payload)?;

    let status = response.status().as_u16();
    let body = response.body_mut().read_to_string()?;
    let body = body.trim().to_string();

    if status >= 400 {
        return Err(AppError::Api { status, body });
    }

    if !body.is_empty() {
        println!("{}", body);
    }

    Ok(())
}

fn resolve_room_id(server: &str, room: &str, access_token: &str) -> Result<String, AppError> {
    if !room.starts_with('#') {
        return Ok(room.to_string());
    }

    let alias = urlencoding::encode(room);
    let url = format!("{}/_matrix/client/v3/directory/room/{}", server, alias);
    let mut response = ureq::get(&url)
        .config()
        .http_status_as_error(false)
        .build()
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .call()?;

    let status = response.status().as_u16();
    let body = response.body_mut().read_to_string()?;
    let body = body.trim().to_string();

    if status >= 400 {
        return Err(AppError::Api { status, body });
    }

    let value: serde_json::Value = serde_json::from_str(&body)?;
    let room_id = value
        .get("room_id")
        .and_then(|room_id| room_id.as_str())
        .ok_or_else(|| AppError::MissingRoomId(body))?;

    Ok(room_id.to_string())
}

fn render_markdown(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = MarkdownParser::new_ext(input, options);
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}
