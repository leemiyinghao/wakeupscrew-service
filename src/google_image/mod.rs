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
    println!("https://www.google.com/search?q={}&espv=2&biw=1366&bih=667&site=webhp&source=lnms&tbm=isch&sa=X&ei=XosDVaCXD8TasATItgE&ved=0CAcQ_AUoAg", utf8_percent_encode(keyword, FRAGMENT).collect::<String>());
    let uri = format!("https://www.google.com/search?q={}&espv=2&biw=1366&bih=667&site=webhp&source=lnms&tbm=isch&sa=X&ei=XosDVaCXD8TasATItgE&ved=0CAcQ_AUoAg", utf8_percent_encode(keyword, FRAGMENT).collect::<String>()).parse().expect("uri encoding fail");
    let page = client.get(uri).await.expect("page fetch fail");
    let buf = hyper::body::to_bytes(page)
        .await
        .expect("page receive fail");
    let page = std::str::from_utf8(&buf).expect("page parse fail");
    let document = Html::parse_document(page);
    // println!("{}", page);
    let selector = Selector::parse(r#"img.t0fcAb"#).unwrap();
    let raw_jsdata = document.select(&selector).next();
    let img_url: &str = raw_jsdata.unwrap().value().attr("src").expect("no src");
    let parent =
        scraper::ElementRef::wrap(raw_jsdata.unwrap().parent().unwrap().parent().unwrap()).unwrap();
    let page_url: &str = parent.value().attr("href").expect("no href");
    Ok(ImageTarget {
        img_url: String::from(img_url),
        page_url: String::from(page_url),
    })
}
async fn download(url: String) -> Result<bytes::Bytes, String> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let uri = url.parse().expect("uri encoding fail");
    let page = client.get(uri).await.expect("page fetch fail");
    let buf = hyper::body::to_bytes(page)
        .await
        .expect("page receive fail");
    Ok(buf)
}
async fn upload(data: bytes::Bytes) -> Result<String, String> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let request = Request::post("https://api.imgur.com/3/image")
        .header("Content-Type", "multipart/form-data")
        .header("Authorization", "Client-ID 6676bc4a87ab89c")
        .body(Body::from(data))
        .expect("construct fail");
    let page = client.request(request).await.expect("page fetch fail");
    let buf = hyper::body::to_bytes(page)
        .await
        .expect("page receive fail");
    println!("{:?}", buf);
    let response: ImgurBasicResponse = serde_json::from_slice(&buf).expect("response parse fail");
    Ok(String::from(response.data.link))
}
pub async fn get(keyword: &str) -> Result<ImageTarget, String> {
    let mut tokit_runtime = Runtime::new().expect("tokio runtime fail");
    let target: ImageTarget = search(keyword).await?;
    let data = download(target.img_url).await?;
    let url = upload(data).await.expect("fail upload");
    Ok(ImageTarget {
        img_url: String::from(url),
        page_url: target.page_url,
    })
}
#[cfg(test)]
use futures::executor::block_on;
#[test]
pub fn test_google_image() {
    let result = block_on(get("修但幾勒")).unwrap();
    println!("{:?}", result);
    assert_eq!(1 + 1, 2);
}
