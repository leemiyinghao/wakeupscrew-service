use hyper::{Body, Client, Request};
extern crate hyper_tls;
use hyper_tls::HttpsConnector;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json;
use tokio::runtime::Runtime;

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
async fn search(keyword: &str) -> Result<ImageTarget, String> {
    //grab search page
    const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri = format!("https://www.google.com/search?q={}&espv=2&biw=1920&bih=966&site=webhp&source=lnms&tbm=isch&sa=X&ei=XosDVaCXD8TasATItgE&ved=0CAcQ_AUoAg", utf8_percent_encode(keyword, FRAGMENT).collect::<String>()).parse();
    if uri.is_err() {
        return Err(String::from("uri parse fail"));
    }
    let page = client.get(uri.unwrap()).await;
    if page.is_err() {
        return Err(String::from("page fetch fail"));
    }
    let buf = hyper::body::to_bytes(page.unwrap()).await;
    if buf.is_err() {
        return Err(String::from("body to bytes fail"));
    }
    let buff = buf.unwrap();
    let page = String::from_utf8_lossy(&buff);
    let document = Html::parse_document(&page[..]);
    let selector = Selector::parse(r#"img.t0fcAb"#).unwrap(); //no need to check static variable
    let raw_jsdata = document.select(&selector).next();
    if raw_jsdata.is_none() {
        return Err(String::from("body to bytes fail"));
    }
    let raw_jsdata_element = raw_jsdata.unwrap();
    let img_url = raw_jsdata_element.value().attr("src");
    let page_url = match raw_jsdata_element.parent() {
        Some(x) => match x.parent() {
            Some(y) => match scraper::ElementRef::wrap(y) {
                Some(z) => match z.value().attr("href") {
                    Some(xx) => Some(xx),
                    None => None,
                },
                None => None,
            },
            None => None,
        },
        None => None,
    };
    if img_url.is_none() || page_url.is_none() {
        return Err(String::from("urls non-exist"));
    }
    Ok(ImageTarget {
        img_url: String::from(img_url.unwrap()),
        page_url: String::from(page_url.unwrap()),
    })
}
async fn download(url: String) -> Result<Box<bytes::Bytes>, String> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri = url.parse().expect("uri encoding fail");
    let page = client.get(uri).await.expect("page fetch fail");
    let buf = hyper::body::to_bytes(page).await.unwrap();
    Ok(Box::new(buf))
}
async fn upload(data: Box<bytes::Bytes>) -> Result<String, String> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
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
        Err(_) => Err(String::from("Imgur response parse fail")),
    }
}
pub async fn get(keyword: &str) -> Result<ImageTarget, String> {
    let target: ImageTarget = search(keyword).await?;
    let data = download(target.img_url).await?;
    let url = upload(data).await?;
    Ok(ImageTarget {
        img_url: String::from(url),
        page_url: target.page_url,
    })
}
#[cfg(test)]
#[test]
pub fn test_google_image() {
    let mut tokit_runtime = Runtime::new().expect("tokio runtime fail");
    let result = tokit_runtime.block_on(get("修但幾勒")).unwrap();
    println!("{:?}", result);
    assert_eq!(1 + 1, 2);
    let result2 = tokit_runtime.block_on(get("https://media.discordapp.net/attachments/483550384133111808/730210148785848390/65656859_2475814475772800_5557129747592380416_n"));
    println!("{:?}", result2);
    assert_eq!(result2.is_err(), true);
}
