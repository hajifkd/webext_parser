use webext_parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", webext_parser::api_pages().await?);
    Ok(())
}
