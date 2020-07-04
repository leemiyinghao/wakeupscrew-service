#[macro_use]
extern crate lazy_static;
use actix_web::{client::Client, post, web, App, HttpResponse, HttpServer, Responder};
use bytes::BytesMut;
use std::env;
mod LineAPI;

#[post("/line")]
async fn line_callback(
    data: web::Json<LineAPI::LineMsg>,
    client: web::Data<Client>,
    auth_token: web::Data<&String>,
) -> impl Responder {
    let event = data.events.get(0).unwrap();
    let text_reply = LineAPI::keyword_switch::switch(&event.message.text[..]);
    if text_reply.is_ok() {
        let reply = LineAPI::LineReply {
            reply_token: event.reply_token.clone(),
            messages: vec![LineAPI::LineReplyMessage {
                r#type: String::from(text_reply.unwrap()),
                text: event.source.user_id.clone(),
            }],
        };

        let mut res = client
            .post("https://api.line.me/v2/bot/message/reply")
            .bearer_auth(auth_token.into_inner())
            .send_json(&reply)
            .await;
    };
    "OK"
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let port: i32 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    let auth_token: String = env::var("AUTH_TOKEN").unwrap();
    HttpServer::new(move || {
        App::new()
            .data(Client::default())
            .data(auth_token.clone())
            .service(line_callback)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
