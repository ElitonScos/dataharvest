use crate::models::Selector;
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector as CssSelector};
use serde_json::{Map, Value};

pub async fn fetch_and_extract(
    client: &Client,
    url: &str,
    selectors: &[Selector],
) -> Result<Value> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (compatible; DataHarvest/1.0)")
        .send()
        .await?;

    let html = response.text().await?;
    let document = Html::parse_document(&html);
    let mut result = Map::new();

    for sel in selectors {
        let css = CssSelector::parse(&sel.css)
            .map_err(|e| anyhow::anyhow!("invalid selector '{}': {:?}", sel.css, e))?;

        let values: Vec<Value> = document
            .select(&css)
            .map(|el| {
                let text = if let Some(attr) = &sel.attribute {
                    el.value().attr(attr).unwrap_or("").to_string()
                } else {
                    el.text().collect::<Vec<_>>().join(" ").trim().to_string()
                };
                Value::String(text)
            })
            .filter(|v| v.as_str().map(|s| !s.is_empty()).unwrap_or(false))
            .collect();

        result.insert(
            sel.field.clone(),
            if values.len() == 1 {
                values.into_iter().next().unwrap()
            } else {
                Value::Array(values)
            },
        );
    }

    Ok(Value::Object(result))
}
