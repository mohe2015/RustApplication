
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    backend::setup("client", |endpoint| async move {
        println!("executing");

        let addr = "[::1]:53794".parse()?;

        println!("jojo");

        let connecting = endpoint.connect(addr, "localhost")?;

        println!("connecting");

        let connected = connecting.await?;
        
        println!("connected");

        Ok(())
    }).await
}