use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "matrix-send",
    version,
    about = "Send a message to a Matrix room"
)]
pub struct Cli {
    #[arg(short, long, value_name = "URL")]
    pub server: String,

    #[arg(short, long, value_name = "ROOM_ID")]
    pub room: String,

    #[arg(short, long, value_name = "TEXT")]
    pub message: String,

    #[arg(short = 't', long, value_enum, default_value_t = MsgType::Markdown)]
    pub msg_type: MsgType,

    #[arg(short = 'a', long, value_name = "PATH")]
    pub access_token_path: String,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum MsgType {
    Text,
    Notice,
    Emote,
    Markdown,
}
