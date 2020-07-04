use std::result::Result;
use regex::Regex;

pub fn switch(keyword: &str) -> Result<&'static str,  &'static str> {
    lazy_static! {
        static ref vec2seq_rule: Regex = Regex::new(r"[^\.]\.\.\.$").unwrap();
        static ref find_image_rule: Regex = Regex::new(r"[^\s]\.jpg$").unwrap();
        static ref wakeup_rule: Regex = Regex::new(r".+醒醒$").unwrap();
    }
    if vec2seq_rule.is_match(keyword) {
        return Ok("系統轉換中，本功能目前無法使用");
    }
    if find_image_rule.is_match(keyword) {
        return Ok("系統轉換中，本功能目前無法使用");
    }
    if wakeup_rule.is_match(keyword) {
        return Ok("系統轉換中，本功能目前無法使用");
    }
    Err("no matched")
}