
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    backend::setup("server", |endpoint| async {

        Ok(())
    }).await
}