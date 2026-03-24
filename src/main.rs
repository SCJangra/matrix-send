mod cli;
mod error;

use clap::Parser;
use cli::{Cli, MsgType};
use error::AppError;
use pulldown_cmark::{html, Options, Parser as MarkdownParser};
use serde_json::json;
use uuid::Uuid;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    send_message(cli)
}

fn send_message(cli: Cli) -> Result<(), AppError> {
    let server = cli.server.trim_end_matches('/');
    let room = urlencoding::encode(&cli.room);
    let access_token = std::fs::read_to_string(&cli.access_token_path)?;
    let access_token = access_token.trim();
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
        .header("Authorization", &format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .send(payload)?;

    let status = response.status().as_u16();
    let body = response.body_mut().read_to_string()?;

    if status >= 400 {
        return Err(AppError::Api { status, body });
    }

    if !body.is_empty() {
        println!("{}", body);
    }

    Ok(())
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
