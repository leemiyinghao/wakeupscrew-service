#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;
use actix::{Actor, StreamHandler};
use actix_web::{
    client::Client, get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use actix_web_actors::ws;
use std::env;
mod line_api;
use std::sync::Arc;
use std::sync::Mutex;

mod google_image;

struct Config {
    auth_token: String,
}
struct WsActor;
impl Actor for WsActor {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn reply(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(WsActor {}, &req, stream);
    resp
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
    let auth_token: String = env::var("AUTH_TOKEN").unwrap();
    let config = Arc::new(Mutex::new(Config {
        auth_token: auth_token,
    }));
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
            .route("/reply/", web::get().to(reply))
            .service(line_callback)
            .service(keepalive)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
