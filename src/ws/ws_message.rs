use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct NickMessage {
    pub nick: String,
}

#[derive(Deserialize)]
pub struct PrivateMessage {
    pub receiver: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct ChatMessage {
    pub message: String,
}

#[derive(Deserialize)]
pub struct HelpMessage {}

#[derive(Deserialize)]
pub struct QuitMessage {}

#[derive(Deserialize)]
#[serde(tag = "message_type")]
pub enum MessageType {
    #[serde(rename = "nick")]
    Nick(NickMessage),
    #[serde(rename = "private")]
    Private(PrivateMessage),
    #[serde(rename = "chat")]
    Chat(ChatMessage),
    #[serde(rename = "help")]
    Help(HelpMessage),
    #[serde(rename = "quit")]
    Quit(QuitMessage),
}
