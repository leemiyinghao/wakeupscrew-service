use log::{debug, info};

use crate::terminator::message::Text;
use actix::fut;
use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web::web;
use actix_web_actors::ws;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::json;
use std::sync::{Arc, Mutex};
use vec2seq_rust::Vec2Seq;

use crate::terminator::message::{
    ChatMessage, JoinRoom, LeaveRoom, ListRooms, SendMessage, WsMessage,
};
use crate::terminator::WsChatServer;

pub struct WsChatSession<'a> {
    pub id: usize,
    pub room: String,
    pub name: String,
    pub iconType: i32,
    pub vec2seq: web::Data<Arc<Mutex<vec2seq_rust::Vec2Seq<'a>>>>,
    pub terminator_threshold: f32,
    pub terminator_self_compare: Option<f32>,
}

impl WsChatSession<'static> {
    pub fn new(
        id: usize,
        room: String,
        name: String,
        iconType: i32,
        vec2seq: web::Data<Arc<Mutex<vec2seq_rust::Vec2Seq<'static>>>>,
        config: web::Data<Arc<Mutex<crate::Config>>>,
    ) -> Self {
        let config = config.lock().unwrap();
        Self {
            id,
            room,
            name,
            iconType,
            vec2seq,
            terminator_threshold: config.terminator_threshold,
            terminator_self_compare: config.terminator_self_compare,
        }
    }
    pub fn join_room(&mut self, room_name: &str, ctx: &mut ws::WebsocketContext<Self>) {
        let room_name = room_name.to_owned();

        // First send a leave message for the current room
        // let leave_msg = LeaveRoom(self.room.clone(), self.name.clone(), self.id);

        // issue_sync comes from having the `BrokerIssue` trait in scope.
        // self.issue_system_sync(leave_msg, ctx);

        // Then send a join message for the new room
        let join_msg = JoinRoom(
            room_name.to_owned(),
            self.name.clone(),
            ctx.address().recipient(),
        );

        WsChatServer::from_registry()
            .send(join_msg)
            .into_actor(self)
            .then(|id, act, _ctx| {
                if let Ok(id) = id {
                    act.room = room_name;
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    pub fn list_rooms(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        WsChatServer::from_registry()
            .send(ListRooms)
            .into_actor(self)
            .then(|res, _, ctx| {
                if let Ok(rooms) = res {
                    for room in rooms {
                        ctx.text(room);
                    }
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    pub fn send_msg(&self, msg: String) {
        let msg = SendMessage(self.room.clone(), self.id, msg);

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(msg);
    }
}

impl Actor for WsChatSession<'static> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.join_room(&self.room.clone()[..], ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let leave_msg = LeaveRoom(self.room.clone(), self.name.clone(), self.id);
        self.issue_system_async(leave_msg);
        info!(
            "WsChatSession closed for {} in room {}",
            self.name.clone(),
            self.room
        );
    }
}

impl Handler<ChatMessage> for WsChatSession<'static> {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession<'static> {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        debug!("WEBSOCKET MESSAGE: {:?}", msg);

        match msg {
            ws::Message::Text(text) => {
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(x) => {
                        // println!("{:?}", x);
                        match x {
                            WsMessage::message(message) => {
                                self.send_msg(text.clone());
                                if message.reliable {
                                    self.send_msg(
                                        json!(WsMessage::received(Text {
                                            text: "bot 判斷中".to_string()
                                        }))
                                        .to_string(),
                                    );
                                    match self.vec2seq.lock() {
                                        Ok(x) => {
                                            let s_m = match x.search_replies(
                                                message.text,
                                                true,
                                                self.terminator_threshold,
                                                self.terminator_self_compare,
                                            ) {
                                                Some(x) => {
                                                    let mut rng = thread_rng();
                                                    let texts = x
                                                        .iter()
                                                        .collect::<Vec<&String>>();
                                                    match texts.choose(&mut rng); {
                                                        Some(text) => WsMessage::reply(Text {
                                                            text: text.to_string(),
                                                        }),
                                                        None => WsMessage::system_message(Text {
                                                            text: "bot 決定跳過".to_string(),
                                                        }),
                                                    }
                                                }
                                                None => WsMessage::system_message(Text {
                                                    text: "bot 決定跳過".to_string(),
                                                }),
                                            };
                                            self.send_msg(json!(s_m).to_string());
                                        }
                                        Err(_) => (), // Err(e) => println!("{:?}", e),
                                    }
                                }
                            }
                            WsMessage::pin(pin) => {
                                self.send_msg(text.clone());
                            }
                            WsMessage::unpin => {
                                self.send_msg(json!(WsMessage::unpin).to_string());
                            }
                            WsMessage::ping => {
                                self.send_msg(json!(WsMessage::pong).to_string());
                            }
                            _ => (),
                        }
                    }
                    Err(e) => {
                        // println!("{:?}: {:?}", e, text);
                    }
                };
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}
