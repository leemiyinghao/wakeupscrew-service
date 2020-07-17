use actix::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct ChatMessage(pub String);

#[derive(Clone, Message)]
#[rtype(result = "usize")]
pub struct JoinRoom(pub String, pub String, pub Recipient<ChatMessage>);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct LeaveRoom(pub String, pub String, pub usize);

#[derive(Clone, Message)]
#[rtype(result = "Vec<String>")]
pub struct ListRooms;

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SendMessage(pub String, pub usize, pub String);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum WsMessage {
    message(Message),
    reply(Text),
    system_message(Text),
    received(Text),
    pin(Message),
    unpin,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub name: String,
    pub iconType: i32,
    pub text: String,
    pub reliable: bool,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Text {
    pub text: String,
}
