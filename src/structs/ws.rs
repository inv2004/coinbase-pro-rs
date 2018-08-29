
#[derive(Serialize, Deserialize, Debug)]
pub struct Subscribe {
    product_ids: Vec<String>,
    channels: Vec<Channel>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Channel {
    Name (String),
    WithProduct {
        name: String,
        product_ids: Vec<String>
    }
}

