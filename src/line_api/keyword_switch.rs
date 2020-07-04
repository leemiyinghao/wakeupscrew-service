use std::result::Result;
use regex::Regex;

pub fn switch(keyword: &str) -> Result<&'static str,  &'static str> {
    lazy_static! {
        static ref VEC2SEQ_RULE: Regex = Regex::new(r"[^\.]\.\.\.$").unwrap();
        static ref FIND_IMAGE_RULE: Regex = Regex::new(r"[^\s]\.jpg$").unwrap();
        static ref WAKEUP_RULE: Regex = Regex::new(r".+醒醒$").unwrap();
    }
    if VEC2SEQ_RULE.is_match(keyword) {
        return Ok("系統轉換中，本功能目前無法使用");
    }
    if FIND_IMAGE_RULE.is_match(keyword) {
        return Ok("系統轉換中，本功能目前無法使用");
    }
    if WAKEUP_RULE.is_match(keyword) {
        return Ok("系統轉換中，本功能目前無法使用");
    }
    Err("no matched")
}