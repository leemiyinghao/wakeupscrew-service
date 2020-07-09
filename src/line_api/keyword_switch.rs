
#[macro_use]
use log;
use env_logger;
use crate::google_image;
use crate::line_api;
use regex::Regex;
use std::result::Result;

pub async fn switch(keyword: &str) -> Result<line_api::LineReply, String> {
    lazy_static! {
        static ref VEC2SEQ_RULE: Regex = Regex::new(r"[^\.]+\.\.\.$").unwrap();
        static ref FIND_IMAGE_RULE: Regex = Regex::new(r"^([^\s]+)\.jpg$").unwrap();
        static ref WAKEUP_RULE: Regex = Regex::new(r".+醒醒$").unwrap();
    }
    if VEC2SEQ_RULE.is_match(keyword) {
        return Ok(line_api::LineReply {
            reply_token: String::from(""),
            messages: vec![line_api::LineMessageType::Text {
                text: String::from("系統轉換中，本功能目前無法使用"),
            }],
        });
    }
    if FIND_IMAGE_RULE.is_match(keyword) {
        let _keyword_iter = FIND_IMAGE_RULE.captures_iter(keyword).next();
        if _keyword_iter.is_none() {
            return Err(String::from("keyword fetch fail"));
        }
        let _keyword = _keyword_iter.unwrap().get(1);
        let image = google_image::get(keyword).await;
        return Ok(match image {
            Ok(x) => line_api::LineReply {
                reply_token: String::from(""),
                messages: vec![line_api::LineMessageType::Image {
                    original_content_url: x.img_url.clone(),
                    preview_image_url: x.img_url.clone(),
                }],
            },
            Err(x) => {
                error!("{:?}", x);
                line_api::LineReply {
                reply_token: String::from(""),
                messages: vec![line_api::LineMessageType::Text {
                    text: String::from("螺絲找不到，螺絲 QQ"),
                }],
            }},
        });
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
