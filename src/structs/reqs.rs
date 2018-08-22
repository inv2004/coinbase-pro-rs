#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub size: f64,
    pub price: f64,
    pub side: String,
    pub product_id: String
}

