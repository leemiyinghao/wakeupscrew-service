use crate::google_image;
use crate::line_api;
use regex::Regex;
use std::result::Result;

pub async fn switch(keyword: &str) -> Result<line_api::LineReply, &'static str> {
    lazy_static! {
        static ref VEC2SEQ_RULE: Regex = Regex::new(r"[^\.]+\.\.\.$").unwrap();
        static ref FIND_IMAGE_RULE: Regex = Regex::new(r"[^\s]\.jpg$").unwrap();
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
        let image = google_image::get(keyword).await;
        if image.is_err() {
            return Ok(line_api::LineReply {
                reply_token: String::from(""),
                messages: vec![line_api::LineMessageType::Text {
                    text: String::from("螺絲找不到，螺絲 QQ"),
                }],
            });
        } else {
            let unwraped = image.unwrap();
            // return Ok(line_api::LineReply {
            //     reply_token: String::from(""),
            //     messages: vec![line_api::LineMessageType::Flex{
            //         alt_text: String::from(keyword),
            //         contents: line_api::SimpleImageFlexContainer{
            //             r#type: String::from("bubble"),
            //             hero: line_api::SimpleImageComponent{
            //                 r#type: String::from("image"),
            //                 url: unwraped.img_url,
            //                 size: String::from("full"),
            //                 aspectRatio: String::from("1:1"),
            //                 aspectMode: String::from("cover"),
            //                 action: line_api::Action{
            //                     r#type: String::from("uri"),
            //                     uri: unwraped.page_url,
            //                 }
            //             }

            //         }
            //     }],
            // });
            let img_url = unwraped.img_url;
            return Ok(line_api::LineReply {
                reply_token: String::from(""),
                messages: vec![line_api::LineMessageType::Image {
                    original_content_url: img_url.clone(),
                    preview_image_url: img_url.clone(),
                }],
            });
        }
    }
    if WAKEUP_RULE.is_match(keyword) {
        return Ok(line_api::LineReply {
            reply_token: String::from(""),
            messages: vec![line_api::LineMessageType::Text {
                text: String::from("系統轉換中，本功能目前無法使用"),
            }],
        });
    }
    Err("no matched")
}
