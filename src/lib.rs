extern crate reqwest;
extern crate scraper;

pub mod api;
pub(crate) mod util;

use scraper::{ElementRef, Html, Selector};
use std::convert::TryFrom;

pub async fn api_pages() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    const BASE: &str = "https://developer.chrome.com/extensions/";

    let api_root = Html::parse_document(&util::get_cached(&format!("{}api_index", BASE)).await?);
    let stable_api_selector =
        Selector::parse("#stable_apis ~ table:nth-of-type(1) tr td:nth-of-type(1) a").unwrap();

    Ok(api_root
        .select(&stable_api_selector)
        .map(|link| link.value().attr("href").unwrap())
        .map(|space| (space.to_owned(), format!("{}{}", BASE, space)))
        .collect())
}

pub async fn parse_apis(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api_root = Html::parse_document(&util::get_cached(url).await?);
    let api_selector = Selector::parse("div.api-reference > *").unwrap();
    let api_info = api_root.select(&api_selector).collect::<Vec<_>>();
    let mut index = 0;

    while index < api_info.len() {
        let title = api_info[index].value();
        let api_type = api::ApiType::try_from(title.id().ok_or("Invalid structure")?)?;
        println!("{:?}", &api_type);
        index += 1;
        let _initial = index;
        while index < api_info.len() && api_info[index].value().name() != "h2" {
            if api_type == api::ApiType::Types {
                println!("{:?}", parse_type(&api_info[index]).unwrap());
            }
            index += 1;
        }
    }

    Ok(())
}

fn parse_type(type_div: &scraper::ElementRef) -> Result<api::Type, String> {
    let name_selector = Selector::parse(r#"h3[id^="type-"]"#).unwrap();
    let name = type_div
        .select(&name_selector)
        .next()
        .ok_or("Invalid structure")?
        .inner_html()
        .trim()
        .to_owned();
    let type_selector = Selector::parse(r#"h3[id^="type-"] ~ table > tbody > tr > th"#).unwrap();
    let type_type = {
        if let Some(th) = type_div.select(&type_selector).next() {
            th.inner_html()
        } else {
            return Ok(api::Type::new_data(name));
        }
    };
    let type_type = type_type.trim();

    match type_type {
        "Enum" => Ok(api::Type::new_enum(name)),
        "properties" => {
            let properties_selector =
                Selector::parse(r#"h3[id^="type-"] ~ table > tbody > tr[id^="property-"]"#)
                    .unwrap();
            let mut properties = vec![];
            let mut optional_properties = vec![];
            for tr in type_div.select(&properties_selector) {
                let tds = tr
                    .children()
                    .filter_map(ElementRef::wrap)
                    .collect::<Vec<_>>();

                if tds.len() != 3 {
                    return Err(format!("Children tds: {} (must be 3)", tds.len()));
                }

                let prop_type = tds[0]
                    .text()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                let prop_td = tds[1];

                if prop_td
                    .select(&Selector::parse("span.optional").unwrap())
                    .count()
                    == 1
                {
                    optional_properties.push(api::Property::new(
                        prop_type,
                        prop_td
                            .text()
                            .nth(1)
                            .ok_or("Invalid structure")?
                            .trim()
                            .to_owned(),
                    ));
                } else {
                    properties.push(api::Property::new(
                        prop_type,
                        prop_td
                            .text()
                            .next()
                            .ok_or("Invalid structure")?
                            .trim()
                            .to_owned(),
                    ));
                }
            }
            Ok(api::Type::new_struct(name, properties, optional_properties))
        }
        "methods" => {
            // TODO
            Ok(api::Type::new_struct(name, vec![], vec![]))
        }
        _ => Err("Invalid type".to_owned()),
    }
}
