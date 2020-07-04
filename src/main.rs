#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate env_logger;
use actix_web::{client::Client, get, middleware, post, web, App, HttpServer, Responder};
use std::env;
mod LineAPI;
use std::sync::Arc;
use std::sync::Mutex;

struct Config {
    auth_token: String,
}

#[post("/line")]
async fn line_callback(
    data: web::Json<LineAPI::LineMsg>,
    client: web::Data<Client>,
    auth_token: web::Data<Arc<Mutex<Config>>>,
) -> impl Responder {
    let event = data.events.get(0).unwrap();
    debug!("{}", event.message.text);
    let text_reply = LineAPI::keyword_switch::switch(&event.message.text[..]);
    if text_reply.is_ok() {
        debug!("text_reply.is_ok: {}", text_reply.unwrap());
        let reply = LineAPI::LineReply {
            reply_token: event.reply_token.clone(),
            messages: vec![LineAPI::LineReplyMessage {
                r#type: String::from("text"),
                text: String::from(text_reply.unwrap()),
            }],
        };
        {
            let mut res = client
                .post("https://api.line.me/v2/bot/message/reply")
                .bearer_auth(&auth_token.lock().unwrap().auth_token)
                .send_json(&reply)
                .await
                .unwrap();
            debug!("connection finish, {}", res.status().as_u16());
            res.body().await.map(move |body_out| {
                debug!(
                    "body: {}",
                    &String::from_utf8(body_out.to_vec()).unwrap()[..]
                )
            });
        }
    } else {
        debug!("{}", text_reply.unwrap_err());
    };
    "OK"
}
#[get("/keepalive")]
async fn keepalive() -> impl Responder {
    debug!("got keepalive");
    LineAPI::keyword_switch::switch("螺絲醒醒").unwrap()
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("server start");
    let port: i32 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    let auth_token: String = env::var("AUTH_TOKEN").unwrap();
    let config = Arc::new(Mutex::new(Config {
        auth_token: auth_token,
    }));
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(config.clone())
            .data(Client::default())
            .service(line_callback)
            .service(keepalive)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
