use hyper::{Body, Client, Request, Uri};
extern crate hyper_tls;
use hyper_tls::HttpsConnector;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json;
use std::fs::File;
use std::io::Write;
use tokio::runtime::Runtime;
#[macro_use]
use log;
use env_logger;
use std::time::Duration;

#[derive(Debug)]
pub struct ImageTarget {
    pub img_url: String,
    pub page_url: String,
}
#[derive(Deserialize, Debug)]
struct ImgurUploadResult {
    link: String,
}
#[derive(Deserialize, Debug)]
struct ImgurBasicResponse {
    data: ImgurUploadResult,
    success: bool,
    status: i64,
}
async fn search(keyword: &str, num: usize) -> Result<ImageTarget, String> {
    //grab search page
    const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
    let https = HttpsConnector::new();
    let client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))
        .build::<_, hyper::Body>(https);
    let uri = format!("https://www.google.com/search?q={}&espv=2&biw=1920&bih=966&site=webhp&source=lnms&tbm=isch&sa=X&ei=XosDVaCXD8TasATItgE&ved=0CAcQ_AUoAg&safe=active", utf8_percent_encode(keyword, FRAGMENT).collect::<String>());
    let mut request = Request::get(uri)
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
    let page = String::from_utf8_lossy(&buff);
    let target_rule =
        regex::Regex::new(r#"\["(https?[^"]+\.(?:jpe?g|JPE?G|png|PNG))",\d+,\d+\]"#).unwrap();
    let m = target_rule.captures(&page[..]);
    if m.is_none() {
        return Err(String::from("img_url not found"));
    }
    let img_url = m.unwrap().get(num + 1);
    match img_url {
        Some(x) => Ok(ImageTarget {
            img_url: String::from(x.as_str()),
            page_url: String::from(x.as_str()),
        }),
        None => Err(String::from("img_url not found")),
    }
}
async fn download(url: String) -> Result<Box<bytes::Bytes>, String> {
    let uri = url.parse().expect("uri encoding fail");
    let page = if url.starts_with("https") {
        let https = HttpsConnector::new();
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build::<_, hyper::Body>(https);
        client.get(uri).await
    } else {
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build_http::<hyper::Body>();
        client.get(uri).await
    };
    if page.is_err() {
        return Err(String::from("image download fail"));
    };
    let buf = hyper::body::to_bytes(page.unwrap()).await.unwrap();
    Ok(Box::new(buf))
}
async fn upload(data: Box<bytes::Bytes>) -> Result<String, String> {
    let https = HttpsConnector::new();
    let client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(30))
        .build::<_, hyper::Body>(https);
    let request = Request::post("https://api.imgur.com/3/image")
        .header("Content-Type", "multipart/form-data")
        .header("Authorization", "Client-ID 6676bc4a87ab89c")
        .body(Body::from(*data))
        .expect("construct fail"); //should not happened
    let page = client.request(request).await;
    if page.is_err() {
        return Err(String::from("upload request fail"));
    };
    let buf = hyper::body::to_bytes(page.unwrap()).await;
    if buf.is_err() {
        return Err(String::from("upload request fail"));
    };
    debug!("{:?}", buf);
    let bbuf = buf.unwrap();
    let response: Result<ImgurBasicResponse, serde_json::Error> = serde_json::from_slice(&bbuf);
    match response {
        Ok(x) => Ok(String::from(x.data.link)),
        Err(x) => Err(format!("Imgur response parse fail: {:?}, {:?}", x, bbuf)),
    }
}
pub async fn get(keyword: &str) -> Result<ImageTarget, String> {
    let mut target: ImageTarget;
    let mut i = 0;
    let url = loop {
        if i > 5 {
            return Err(String::from("not found"));
        };
        target = search(keyword, i).await?;
        i += 1;
        debug!("{:?}", target.img_url);
        let data = match download(target.img_url).await {
            Ok(x) => x,
            Err(_) => continue,
        };
        match upload(data).await {
            Ok(x) => break x,
            Err(_) => continue,
        }
    };
    Ok(ImageTarget {
        img_url: String::from(url),
        page_url: target.page_url,
    })
}
#[cfg(test)]
#[test]
pub fn test_google_image() {
    let mut tokit_runtime = Runtime::new().expect("tokio runtime fail");
    let result = tokit_runtime
        .block_on(get("今日も一日がんばるぞい"))
        .unwrap();
    println!("{:?}", result);
    assert_eq!(1 + 1, 2);
    let result2 = tokit_runtime.block_on(get("https://media.discordapp.net/attachments/483550384133111808/730210148785848390/65656859_2475814475772800_5557129747592380416_n"));
    println!("{:?}", result2);
    assert_eq!(result2.is_err(), true);
}
