
#[derive(Serialize, Deserialize, Debug)]
pub struct Subscribe {
    #[serde(rename = "type")]
    pub _type: SubscribeCmd,
    pub product_ids: Vec<String>,
    pub channels: Vec<Channel>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SubscribeCmd {
    Subscribe
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Channel {
    Name (ChannelType),
    WithProduct {
        name: ChannelType,
        product_ids: Vec<String>
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ChannelType {
    Heartbeat,
    Ticker,
    Level2,
    User,
    Matches
}

