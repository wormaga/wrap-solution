use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub price: i64,
    pub rating: f64,
    pub stock: i64,
    pub brand: String,
    pub category: String,
    pub thumbnail: String,
    pub images: Vec<String>,
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let product: Product = Client::new()
        .get("https://dummyjson.com/products/1")
        .send()
        .await?
        .json()
        .await?;
    println!("{:#?}", product);
    Ok(())
}
