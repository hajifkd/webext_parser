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

pub async fn parse_apis(
    name: &str,
    url: &str,
) -> Result<api::Namespace, Box<dyn std::error::Error>> {
    let api_root = Html::parse_document(&util::get_cached(url).await?);
    let api_selector = Selector::parse("div.api-reference > *").unwrap();
    let api_info = api_root.select(&api_selector).collect::<Vec<_>>();
    let mut index = 0;
    let mut types = vec![];
    let mut methods = vec![];
    let mut events = vec![];
    let mut properties = vec![];

    while index < api_info.len() {
        let title = api_info[index].value();
        let api_type = api::ApiType::try_from(title.id().ok_or("Invalid API structure")?)?;
        index += 1;
        let _initial = index;
        while index < api_info.len() && api_info[index].value().name() != "h2" {
            match api_type {
                api::ApiType::Types => {
                    if let Ok(t) = parse_type(api_info[index]) {
                        types.push(t);
                    }
                }
                api::ApiType::Methods => {
                    if let Ok(m) = parse_method(api_info[index], "h3") {
                        methods.push(m);
                    }
                }
                api::ApiType::Events => {
                    if let Ok(e) = parse_event(api_info[index]) {
                        events.push(e);
                    }
                }
                api::ApiType::Properties => {
                    if let Ok(ps) = parse_properties(api_info[index]) {
                        properties = ps;
                    }
                }
            }
            index += 1;
        }
    }

    Ok(api::Namespace::new(
        name.to_owned(),
        types,
        properties,
        methods,
        events,
    ))
}

struct ParsedElem<'a> {
    type_name: String,
    val_name: String,
    optional: bool,
    desc_col: Option<scraper::ElementRef<'a>>,
}

fn parse_elem<'a>(tr: scraper::ElementRef<'a>) -> Result<ParsedElem<'a>, String> {
    let tds = tr
        .children()
        .filter_map(ElementRef::wrap)
        .collect::<Vec<_>>();
    if tds.len() < 2 {
        return Err(format!("Children tds: {} (must be 3)", tds.len()));
    }

    let desc_col = if tds.len() == 3 { Some(tds[2]) } else { None };

    let prop_type = tds[0]
        .text()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let prop_td = tds[1];
    let optional = prop_td
        .select(&Selector::parse("span.optional").unwrap())
        .count()
        == 1;

    Ok(ParsedElem {
        type_name: prop_type,
        val_name: prop_td
            .text()
            .nth(if optional { 1 } else { 0 })
            .ok_or("Invalid Element structure")?
            .trim()
            .to_owned(),
        optional,
        desc_col: desc_col,
    })
}

fn parse_name(div: scraper::ElementRef, title_selector: &str) -> Result<String, String> {
    let name_selector = Selector::parse(&format!(r#"{}"#, title_selector)).unwrap();
    match util::take_one(div.select(&name_selector)) {
        util::TakeResult::One(e) => Ok(e.inner_html().trim().to_owned()),
        util::TakeResult::Zero => Err("No name found".to_owned()),
        util::TakeResult::More => Err("Multiple names found".to_owned()),
    }
}

fn parse_type(type_div: scraper::ElementRef) -> Result<api::Type, String> {
    let name = parse_name(type_div, r#"h3[id^="type-"]"#)?;
    let tr_selector = Selector::parse(r#"h3[id^="type-"] ~ table > tbody > tr"#).unwrap();
    let trs = type_div.select(&tr_selector).collect::<Vec<_>>();
    if trs.len() == 0 {
        return Ok(api::Type::new_data(name));
    }

    let mut index = 0;
    let mut methods = vec![];
    let mut properties = vec![];
    let mut optional_properties = vec![];
    let mut events = vec![];

    while index < trs.len() {
        let tr = trs[index];
        let prop_type = match util::take_one(
            tr.children()
                .filter_map(ElementRef::wrap)
                .filter(|e| e.value().name() == "th"),
        ) {
            util::TakeResult::One(e) => e.inner_html(),
            _ => return Err("Invalid type header".to_owned()),
        };
        let prop_type = prop_type.trim();
        index += 1;
        let start_index = index;
        while index < trs.len()
            && trs[index]
                .children()
                .filter_map(ElementRef::wrap)
                .filter(|e| e.value().name() == "th")
                .count()
                == 0
        {
            index += 1;
        }

        match prop_type {
            "Enum" => return Ok(api::Type::new_enum(name)),
            "properties" => {
                for tr in &trs[start_index..index] {
                    let elem = parse_elem(*tr)?;
                    if elem.desc_col.is_none() {
                        return Err("Children tds must be 3".to_owned());
                    }
                    if elem.optional {
                        optional_properties.push(api::Element::new(elem.type_name, elem.val_name));
                    } else {
                        properties.push(api::Element::new(elem.type_name, elem.val_name));
                    }
                }
            }
            "methods" => {
                for tr in &trs[start_index..index] {
                    let method_div = match util::take_one(
                        tr.children()
                            .filter_map(ElementRef::wrap)
                            .filter(|e| e.value().name() == "td"),
                    ) {
                        util::TakeResult::One(td) => match util::take_one(
                            td.children()
                                .filter_map(ElementRef::wrap)
                                .filter(|e| e.value().name() == "div"),
                        ) {
                            util::TakeResult::One(div) => div,
                            _ => return Err("div not found in Type".to_owned()),
                        },
                        _ => return Err("td not found in Type".to_owned()),
                    };
                    methods.push(parse_method(method_div, "h4")?);
                }
            }
            "events" => {
                for tr in &trs[start_index..index] {
                    let event_div = match util::take_one(
                        tr.children()
                            .filter_map(ElementRef::wrap)
                            .filter(|e| e.value().name() == "td"),
                    ) {
                        util::TakeResult::One(td) => match util::take_one(
                            td.children()
                                .filter_map(ElementRef::wrap)
                                .filter(|e| e.value().name() == "div"),
                        ) {
                            util::TakeResult::One(div) => div,
                            _ => return Err("div not found in Event".to_owned()),
                        },
                        _ => return Err("td not found in Event".to_owned()),
                    };
                    events.push(parse_inner_event(event_div)?);
                }
            }
            _ => return Err("Invalid type".to_owned()),
        }
    }

    Ok(api::Type::new_struct(
        name,
        properties,
        optional_properties,
        methods,
        events,
    ))
}

fn parse_event(event_div: scraper::ElementRef) -> Result<api::Event, String> {
    let method = parse_method(event_div, "div.description > div > h4")?;
    let name = parse_name(event_div, r#"h3[id^="event-"]"#)?;
    Ok(api::Event::new(name, method))
}

fn parse_inner_event(event_div: scraper::ElementRef) -> Result<api::Event, String> {
    let method = parse_method(event_div, "h4")?;
    let name_selector = Selector::parse("div.summary > code.prettyprint").unwrap();
    let name = event_div
        .select(&name_selector)
        .next()
        .ok_or("Invalid event name structure".to_owned())?
        .inner_html()
        .trim()
        .split('.')
        .next()
        .ok_or("Invalid event code structure".to_owned())?
        .to_owned();
    Ok(api::Event::new(name, method))
}

fn parse_method(
    method_div: scraper::ElementRef,
    title_selector: &str,
) -> Result<api::Method, String> {
    let name = parse_name(method_div, title_selector)?;
    let tbody_selector = Selector::parse(&format!(
        r#"{} ~ div.description > table > tbody"#,
        title_selector
    ))
    .unwrap();

    let arguments = match util::take_one(method_div.select(&tbody_selector)) {
        util::TakeResult::Zero => vec![],
        util::TakeResult::One(e) => parse_method_body(e)?,
        _ => return Err("Unsupported method len".to_owned()),
    };

    Ok(api::Method::new(name, arguments))
}

fn parse_method_body(args_tbody: scraper::ElementRef) -> Result<Vec<api::Argument>, String> {
    let mut result = vec![];
    for tr in args_tbody
        .children()
        .filter_map(ElementRef::wrap)
        .filter(|&e| e.value().id().is_some())
    {
        let elem = parse_elem(tr)?;

        let arg = if elem.type_name == "function" {
            let tbody = elem
                .desc_col
                .ok_or("No info for callback found".to_owned())?
                .children()
                .filter_map(ElementRef::wrap)
                .filter(|e| e.value().name() == "table")
                .map(|t| {
                    match util::take_one(
                        t.children()
                            .filter_map(ElementRef::wrap)
                            .filter(|e| e.value().name() == "tbody"),
                    ) {
                        util::TakeResult::One(e) => e,
                        _ => panic!("tbody not found in method"),
                    }
                });
            let callback_args = match util::take_one(tbody) {
                util::TakeResult::Zero => vec![],
                util::TakeResult::One(tbody) => parse_method_body(tbody)?,
                _ => return Err("Multiple argument info found".to_owned()),
            };
            let method = api::Method::new(elem.val_name, callback_args);
            api::Argument::new_callback(method, elem.optional)
        } else {
            let element = api::Element::new(elem.type_name, elem.val_name);
            api::Argument::new_element(element, elem.optional)
        };

        result.push(arg);
    }

    Ok(result)
}

fn parse_properties(prop_table: scraper::ElementRef) -> Result<Vec<api::Property>, String> {
    let tbody = match util::take_one(prop_table.children().filter_map(ElementRef::wrap)) {
        util::TakeResult::One(tbody) => tbody,
        util::TakeResult::More => return Err("Multiple tbody found in Properties".to_owned()),
        util::TakeResult::Zero => return Err("No tbody found in Properties".to_owned()),
    };

    let mut result = vec![];

    for tr in tbody.children().filter_map(ElementRef::wrap) {
        let elem = parse_elem(tr)?;
        if elem.optional {
            return Err("Properties cannot be optional".to_owned());
        }
        if elem.type_name != "object" {
            let type_name = if elem
                .type_name
                .chars()
                .next()
                .map(|c| !c.is_ascii_alphabetic())
                .unwrap_or(false)
            {
                if elem.type_name.contains('.') {
                    "number"
                } else {
                    "integer"
                }
            } else {
                &elem.type_name
            };

            result.push(api::Property::new_immediate(
                elem.val_name,
                type_name.to_owned(),
            ));
        } else {
            if let Some(td) = elem.desc_col {
                let tbody = match util::take_one(
                    td.children()
                        .filter_map(ElementRef::wrap)
                        .filter(|e| e.value().name() == "table")
                        .flat_map(|t| {
                            t.children()
                                .filter_map(ElementRef::wrap)
                                .filter(|e| e.value().name() == "tbody")
                        }),
                ) {
                    util::TakeResult::One(td) => td,
                    util::TakeResult::Zero => {
                        panic!("no table found");
                        // result.push(api::Property::new_immediate(elem.val_name, elem.type_name));
                        // continue;
                    }
                    util::TakeResult::More => {
                        panic!("too many table found");
                    }
                };

                let mut methods = vec![];
                let mut flag = false;
                for tr in tbody.children().filter_map(ElementRef::wrap) {
                    let method = parse_method(tr, r#"h3[id^="method-"]"#);
                    if method.is_err() {
                        flag = true;
                        break;
                    }
                    methods.push(method.unwrap());
                }

                if flag {
                    result.push(api::Property::new_immediate(elem.val_name, elem.type_name));
                    continue;
                }

                result.push(api::Property::new_object(elem.val_name, methods));
            } else {
                result.push(api::Property::new_immediate(elem.val_name, elem.type_name));
            }
        }
    }

    Ok(result)
}
