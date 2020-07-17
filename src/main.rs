#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;
use actix::prelude::*;
use actix::*;
use actix::{Actor, Addr, StreamHandler};
use actix_web::{
    client::Client, get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use actix_web_actors::ws;
use actix_web_actors::ws::{Message, WebsocketContext};
use std::env;
mod line_api;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

mod google_image;
mod terminator;
use terminator::{session::WsChatSession, WsChatServer};

struct Config {
    auth_token: String,
}

async fn reply<'a>(
    info: web::Path<(String, String, i32)>,
    req: HttpRequest,
    stream: web::Payload,
    vec2seq: web::Data<Arc<Mutex<vec2seq_rust::Vec2Seq<'static>>>>,
) -> Result<HttpResponse, Error> {
    let session = WsChatSession::new(0, info.0.clone(), info.1.clone(), info.2, vec2seq.clone());
    ws::start(session, &req, stream)
}

#[post("/line")]
async fn line_callback(
    data: web::Json<line_api::LineMsg>,
    client: web::Data<Client>,
    auth_token: web::Data<Arc<Mutex<Config>>>,
    vec2seq: web::Data<Arc<Mutex<vec2seq_rust::Vec2Seq<'_>>>>,
) -> impl Responder {
    let event = data.events.get(0).unwrap();
    info!("{}", event.message.text);
    //group history memorizer
    let _reply =
        line_api::keyword_switch::switch(&event.message.text[..], &vec2seq.lock().unwrap()).await;
    info!("{:?}", _reply);
    if _reply.is_ok() {
        let reply = line_api::LineReply {
            reply_token: event.reply_token.clone(),
            messages: _reply.unwrap().messages,
        };
        {
            let mut res = client
                .post("https://api.line.me/v2/bot/message/reply")
                .bearer_auth(&auth_token.lock().unwrap().auth_token)
                .send_json(&reply)
                .await;
            if res.is_err() {
                error!("connect fail: {}", res.unwrap_err());
            }
        }
    };
    "OK"
}
#[get("/keepalive")]
async fn keepalive() -> impl Responder {
    debug!("got keepalive");
    "i'm alive"
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("server start");
    let port: i32 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    let host: String = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let auth_token: String = match env::var("AUTH_TOKEN") {
        Ok(x) => x,
        Err(_) => {
            error!("server runs without line api key: AUTH_TOKEN, set one at .env or shell env.");
            "".to_string()
        }
    };
    let config = Arc::new(Mutex::new(Config {
        auth_token: auth_token,
    }));
    let vec2seq = Arc::new(Mutex::new(vec2seq_rust::Vec2Seq::new(
        std::path::Path::new("../cut_corpus/finalfusion.10e.w_zh_en_ptt.s60.pq.fifu"),
        std::path::Path::new("../vec2seq_rust/tfidf.bin"),
        std::path::Path::new("../vec2seq_rust/stopwords.txt"),
        std::path::Path::new("../vec2seq_rust/reply_group.index.granne"),
        std::path::Path::new("../vec2seq_rust/reply.index.granne"),
        std::path::Path::new("../vec2seq_rust/reply_group.element.granne"),
        std::path::Path::new("../vec2seq_rust/reply.element.granne"),
        std::path::Path::new("../vec2seq_rust/db/reply_group"),
    )));
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(config.clone())
            .data(Client::default())
            .data(vec2seq.clone())
            .route("/reply/{room}/{name}/{iconType}", web::get().to(reply))
            .service(line_callback)
            .service(keepalive)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
