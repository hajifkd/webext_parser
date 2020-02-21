extern crate reqwest;
extern crate scraper;

use scraper::{Html, Selector};

pub async fn api_pages() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    const BASE: &str = "https://developer.chrome.com/extensions/";

    let api_root = Html::parse_document(
        &reqwest::get(&format!("{}api_index", BASE))
            .await?
            .text()
            .await?,
    );
    let stable_api_selector =
        Selector::parse("#stable_apis ~ table:nth-of-type(1) tr td:nth-of-type(1) a").unwrap();

    Ok(api_root
        .select(&stable_api_selector)
        .map(|api| format!("{}{}", BASE, api.value().attr("href").unwrap()))
        .collect())
}
