use webext_parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for (space, url) in webext_parser::api_pages().await?.iter() {
        /* if space != "declarativeContent" {
            continue;
        } */
        println!("{}", space);
        webext_parser::parse_apis(&url).await?;
    }
    Ok(())
}
