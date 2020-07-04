use serde::{Serialize, Deserialize};
pub mod keyword_switch;

#[derive(Debug, Serialize, Deserialize)]
pub struct LineMessage {
    pub r#type: String,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineReplyMessage {
    pub r#type: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineSource {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineEvent {
    pub r#type: String,
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    pub source: LineSource,
    pub timestamp: i64,
    pub mode: String,
    pub message: LineMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineMsg {
    pub events: Vec<LineEvent>,
    pub destination: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineReply {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    pub messages: Vec<LineReplyMessage>,
}