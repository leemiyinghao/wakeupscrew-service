use serde::{Deserialize, Serialize};
pub mod keyword_switch;
// pub mod flex_message;

#[derive(Debug, Serialize, Deserialize)]
pub struct LineMessage {
    pub r#type: String,
    pub id: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineSource {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineEvent {
    pub r#type: String,
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    pub source: LineSource,
    pub timestamp: i64,
    pub mode: String,
    pub message: LineMessage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineMsg {
    pub events: Vec<LineEvent>,
    pub destination: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineReply {
    #[serde(rename = "replyToken")]
    pub reply_token: String,
    pub messages: Vec<LineMessageType>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum LineMessageType {
    Text {
        text: String,
    },
    Image {
        #[serde(default, rename = "originalContentUrl")]
        original_content_url: String,
        #[serde(default, rename = "previewImageUrl")]
        preview_image_url: String,
    },
    Video {
        #[serde(default, rename = "originalContentUrl")]
        original_content_url: String,
        #[serde(default, rename = "previewImageUrl")]
        preview_image_url: String,
    },
    Audio {
        #[serde(default, rename = "originalContentUrl")]
        original_content_url: String,
        #[serde(default)]
        duration: u64,
    },
    Location {
        #[serde(default)]
        title: String,
        address: String,
        latitude: f64,
        longitude: f64,
    },
    File {
        #[serde(rename = "fileName")]
        file_name: String,
        #[serde(rename = "fileSize")]
        file_size: u64,
    },
    Sticker {
        #[serde(rename = "packageId")]
        package_id: String,
        #[serde(rename = "stickerId")]
        sticker_id: String,
    },
    Flex {
        #[serde(default, rename = "altText")]
        alt_text: String,
        contents: SimpleImageFlexContainer,
    },
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleImageFlexContainer {
    r#type: String,
    hero: SimpleImageComponent,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SimpleImageComponent {
    r#type: String,
    url: String,
    size: String,
    aspectRatio: String,
    aspectMode: String,
    action: Action,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Action {
    r#type: String,
    uri: String,
}
