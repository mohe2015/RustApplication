pub mod lib;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    self::lib::setup().await
}