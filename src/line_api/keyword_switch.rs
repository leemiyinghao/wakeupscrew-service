#[macro_use]
use log;
use crate::google_image;
use crate::line_api;
use env_logger;
use hyper::{Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use std::result::Result;

pub async fn switch(
    keyword: &str,
    vec2seq: &vec2seq_rust::Vec2Seq<'_>,
    linebot_threshold: f32,
    linebot_self_compare: Option<f32>,
    terminator_threshold: f32,
    terminator_self_compare: Option<f32>,
    imgur_auth_token: &str,
) -> Result<line_api::LineReply, String> {
    lazy_static! {
        static ref VEC2SEQ_RULE: Regex = Regex::new(r"([^\.]+)(?:\.\.\.|…|⋯)$").unwrap();
        static ref FIND_IMAGE_RULE: Regex = Regex::new(r"^([^\s]+)\.(?:jpg|png|PNG|JPG)$").unwrap();
        static ref CONVERT_IMAGE_RULE: Regex =
            Regex::new(r"^https([^\s]+)\.(?:jpg|png|PNG|JPG)$").unwrap();
        static ref FIND_WEB_RULE: Regex = Regex::new(r"^https?([^\s]+)$").unwrap();
        static ref WEB_TITLE_RULE: Regex = Regex::new(r"<title>(.*?)</title>").unwrap();
        static ref WAKEUP_RULE: Regex = Regex::new(r".+醒醒$").unwrap();
    }
    if VEC2SEQ_RULE.is_match(keyword) {
        let _keyword_iter = VEC2SEQ_RULE.captures_iter(keyword).next();
        if _keyword_iter.is_none() {
            return Err(String::from("keyword fetch fail"));
        }
        let _keyword = _keyword_iter.unwrap().get(1);
        if _keyword.is_none() {
            return Err(String::from("keyword fetch fail"));
        }
        let replies = match vec2seq.search_replies(
            String::from(_keyword.unwrap().as_str()),
            true,
            linebot_threshold,
            linebot_self_compare,
        ) {
            Some(x) => x,
            None => vec![String::from("阿哈哈，螺絲不知道")],
        };
        let mut rng = thread_rng();
        return Ok(line_api::LineReply {
            reply_token: String::from(""),
            messages: vec![line_api::LineMessageType::Text {
                text: replies.choose(&mut rng).unwrap().to_string(),
            }],
        });
    }
    if FIND_IMAGE_RULE.is_match(keyword) {
        if CONVERT_IMAGE_RULE.is_match(keyword) {
            return Ok(line_api::LineReply {
                reply_token: String::from(""),
                messages: vec![line_api::LineMessageType::Image {
                    original_content_url: keyword.to_string().clone(),
                    preview_image_url: keyword.to_string().clone(),
                }],
            });
        }
        let _keyword_iter = FIND_IMAGE_RULE.captures_iter(keyword).next();
        if _keyword_iter.is_none() {
            return Err(String::from("keyword fetch fail"));
        }
        let _keyword = _keyword_iter.unwrap().get(1);
        if _keyword.is_none() {
            return Err(String::from("keyword fetch fail"));
        }
        let image = google_image::get(_keyword.unwrap().as_str(), imgur_auth_token).await;
        return Ok(match image {
            Ok(x) => line_api::LineReply {
                reply_token: String::from(""),
                messages: vec![line_api::LineMessageType::Image {
                    original_content_url: x.img_url.clone(),
                    preview_image_url: x.img_url.clone(),
                }],
            },
            Err(x) => {
                warn!("{:?}", x);
                line_api::LineReply {
                    reply_token: String::from(""),
                    messages: vec![line_api::LineMessageType::Text {
                        text: String::from("螺絲找不到，螺絲 QQ"),
                    }],
                }
            }
        });
    }
    //url auto trace
    if FIND_WEB_RULE.is_match(keyword) {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let mut request = Request::get(keyword)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
            )
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:78.0) Gecko/20100101 Firefox/78.0",
            )
            .body(Body::from(bytes::Bytes::new()));
        if request.is_err() {
            return Err(String::from("uri parse fail"));
        }
        let page = client.request(request.unwrap()).await;
        if page.is_err() {
            return Err(String::from("page fetch fail"));
        }
        let buf = hyper::body::to_bytes(page.unwrap()).await;
        if buf.is_err() {
            debug!("{:?}", buf);
            return Err(String::from("body to bytes fail"));
        }
        let buff = buf.unwrap();
        // let mut output = File::create("result.html").unwrap();
        // output.write(&buff).unwrap();
        let page = String::from_utf8_lossy(&buff).into_owned();
        let _keyword_iter = WEB_TITLE_RULE.captures_iter(page.as_str()).next();
        if _keyword_iter.is_none() {
            return Err(String::from("keyword fetch fail"));
        }
        let _keyword = _keyword_iter.unwrap().get(1);
        if _keyword.is_none() {
            return Err(String::from("keyword fetch fail"));
        }
        match vec2seq.search_replies(
            String::from(_keyword.unwrap().as_str()),
            true,
            terminator_threshold,
            terminator_self_compare,
        ) {
            Some(replies) => {
                let mut rng = thread_rng();
                return Ok(line_api::LineReply {
                    reply_token: String::from(""),
                    messages: vec![line_api::LineMessageType::Text {
                        text: replies.choose(&mut rng).unwrap().to_string(),
                    }],
                });
            }
            None => (),
        };
    }
    if WAKEUP_RULE.is_match(keyword) {
        return Ok(line_api::LineReply {
            reply_token: String::from(""),
            messages: vec![line_api::LineMessageType::Text {
                text: String::from("系統轉換中，本功能目前無法使用"),
            }],
        });
    }
    Err(String::from("no matched"))
}
