#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate dotenv_codegen;
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

#[derive(Debug)]
pub struct Config {
    auth_token: String,
    linebot_threshold: f32,
    linebot_self_compare: Option<f32>,
    terminator_threshold: f32,
    terminator_self_compare: Option<f32>,
}

async fn reply<'a>(
    info: web::Path<(String, String, i32)>,
    req: HttpRequest,
    stream: web::Payload,
    vec2seq: web::Data<Arc<Mutex<vec2seq_rust::Vec2Seq<'static>>>>,
    config: web::Data<Arc<Mutex<Config>>>,
) -> Result<HttpResponse, Error> {
    let session = WsChatSession::new(
        0,
        info.0.clone(),
        info.1.clone(),
        info.2,
        vec2seq.clone(),
        config,
    );
    ws::start(session, &req, stream)
}

#[post("/line")]
async fn line_callback(
    data: web::Json<line_api::LineMsg>,
    client: web::Data<Client>,
    config: web::Data<Arc<Mutex<Config>>>,
    vec2seq: web::Data<Arc<Mutex<vec2seq_rust::Vec2Seq<'_>>>>,
) -> impl Responder {
    let event = data.events.get(0).unwrap();
    info!("{}", event.message.text);
    let guard = config.lock().unwrap();
    //group history memorizer
    let _reply = line_api::keyword_switch::switch(
        &event.message.text[..],
        &vec2seq.lock().unwrap(),
        guard.linebot_threshold,
        guard.linebot_self_compare,
    )
    .await;
    info!("{:?}", _reply);
    if _reply.is_ok() {
        let reply = line_api::LineReply {
            reply_token: event.reply_token.clone(),
            messages: _reply.unwrap().messages,
        };
        {
            let mut res = client
                .post("https://api.line.me/v2/bot/message/reply")
                .bearer_auth(&guard.auth_token)
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
    let linebot_threshold: f32 = match env::var("LINEBOT_THRESHOLD") {
        Ok(x) => x.parse().unwrap_or(0.75f32),
        Err(_) => 0.75f32,
    };
    let linebot_self_compare: Option<f32> = match env::var("LINEBOT_SELF_COMPARE") {
        Ok(x) => match x.parse() {
            Ok(a) => Some(a),
            Err(_) => None,
        },
        Err(_) => None,
    };
    let terminator_threshold: f32 = match env::var("TERMINATOR_THRESHOLD") {
        Ok(x) => x.parse().unwrap_or(0.75f32),
        Err(_) => 0.75f32,
    };
    let terminator_self_compare: Option<f32> = match env::var("TERMINATOR_SELF_COMPARE") {
        Ok(x) => match x.parse() {
            Ok(a) => Some(a),
            Err(_) => None,
        },
        Err(_) => Some(0.2f32),
    };
    let config = Arc::new(Mutex::new(Config {
        auth_token: auth_token,
        linebot_threshold,
        linebot_self_compare,
        terminator_threshold,
        terminator_self_compare,
    }));
    println!("{:?}", config);
    let vec2seq = Arc::new(Mutex::new(vec2seq_rust::Vec2Seq::new(
        std::path::Path::new("finalfusion.10e.w_zh_en_ptt.s60.pq.fifu"),
        std::path::Path::new("tfidf.bin"),
        std::path::Path::new("stopwords.txt"),
        std::path::Path::new("reply_group.index.granne"),
        std::path::Path::new("reply.index.granne"),
        std::path::Path::new("reply_group.element.granne"),
        std::path::Path::new("reply.element.granne"),
        std::path::Path::new("db/reply_group"),
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
