use std::collections::HashMap;

use anyhow::{anyhow, Result};
use boa_engine::{object::builtins::JsArray, Context, JsResult, JsValue, Source};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize)]
pub struct Property {
    pub id: u32,
    pub latitude: f64,
    pub longitude: f64,
    pub price: u32,
    pub bedrooms: i8,
    pub shared: bool,
    pub studio: bool,
    pub live: bool,
    pub furnished: bool,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PropertyDetails {
    pub id: usize,
    pub title: String,
    pub image_url: String,
    pub last_updated: String,
    pub description: String,
    pub details: Vec<String>,
    pub rent_per_month: f32,
}

pub async fn get_properties(longitude: f64, latitude: f64, area: u32) -> Result<Vec<Property>> {
    let script = fetch_script(longitude, latitude, area).await?;
    parse_properties(script)
}

async fn fetch_script(longitude: f64, latitude: f64, area: u32) -> Result<String> {
    let response = reqwest::get(format!(
        "https://www.openrent.co.uk/search/search_bycommutetime?lngn={}&latn={}&area={}",
        longitude, latitude, area
    ))
    .await?;
    let document = scraper::Html::parse_document(&response.text().await?);
    let script_selector = scraper::Selector::parse(r#"script[type="text/javascript"]"#).unwrap();
    let script = document
        .select(&script_selector)
        .find(|e| {
            e.inner_html()
                .contains("// Initialise Variables for search js")
        })
        .ok_or(anyhow!("couldn't find main script"))?;
    Ok(script.inner_html())
}

fn parse_properties(script: String) -> Result<Vec<Property>> {
    let mut context = Context::default();

    js_eval(&mut context, script)?;
    let ids = js_eval_array(&mut context, "PROPERTYIDS")?;
    let longitudes = js_eval_array(&mut context, "PROPERTYLISTLONGITUDES")?;
    let latitudes = js_eval_array(&mut context, "PROPERTYLISTLATITUDES")?;
    let prices = js_eval_array(&mut context, "prices")?;
    let bedrooms = js_eval_array(&mut context, "bedrooms")?;
    let studio = js_eval_array(&mut context, "isstudio")?;
    let shared = js_eval_array(&mut context, "isshared")?;
    let live = js_eval_array(&mut context, "islivelistBool")?;
    let furnished = js_eval_array(&mut context, "furnished")?;

    (0..js_len(&mut context, &ids)?)
        .map(|i| {
            let id = js_at_u32(&mut context, &ids, i)?;
            Ok(Property {
                id,
                longitude: js_at_f64(&mut context, &longitudes, i)?,
                latitude: js_at_f64(&mut context, &latitudes, i)?,
                price: js_at_u32(&mut context, &prices, i)?,
                bedrooms: js_at_i8(&mut context, &bedrooms, i)?,
                studio: js_at_i8(&mut context, &studio, i)? == 1,
                shared: js_at_i8(&mut context, &shared, i)? == 1,
                live: js_at_i8(&mut context, &live, i)? == 1,
                furnished: js_at_i8(&mut context, &furnished, i)? == 1,
                url: format!("https://www.openrent.co.uk/{}", id),
            })
        })
        .collect()
}

fn js_eval<S: AsRef<[u8]>>(context: &mut Context, script: S) -> Result<JsValue> {
    js_err(context.eval(Source::from_bytes(&script)))
}

fn js_eval_array<S: AsRef<[u8]>>(context: &mut Context, script: S) -> Result<JsArray> {
    let object = js_eval(context, script)?
        .as_object()
        .ok_or(anyhow!("invalid array"))?
        .clone();
    js_err(JsArray::from_object(object))
}

fn js_err<T>(res: JsResult<T>) -> Result<T> {
    res.map_err(|e| anyhow!("{}", e))
}

fn js_len(context: &mut Context, array: &JsArray) -> Result<u32> {
    js_err(array.length(context).map(|l| l as u32))
}

fn js_at(context: &mut Context, array: &JsArray, index: u32) -> Result<JsValue> {
    js_err(array.at(index as i64, context))
}

fn js_at_u32(context: &mut Context, array: &JsArray, index: u32) -> Result<u32> {
    js_err(js_at(context, array, index)?.to_u32(context))
}

fn js_at_i8(context: &mut Context, array: &JsArray, index: u32) -> Result<i8> {
    js_err(js_at(context, array, index)?.to_int8(context))
}

fn js_at_f64(context: &mut Context, array: &JsArray, index: u32) -> Result<f64> {
    js_at(context, array, index)?
        .as_number()
        .ok_or(anyhow!("invalid longitude"))
}

pub async fn get_property_details(ids: Vec<usize>) -> Result<Vec<PropertyDetails>> {
    let mut url = Url::parse("https://www.openrent.co.uk/search/propertiesbyid")?;
    for id in &ids {
        url.query_pairs_mut().append_pair("ids", &id.to_string());
    }
    let mut property_details_map: HashMap<usize, PropertyDetails> = reqwest::get(url)
        .await?
        .json::<Vec<PropertyDetails>>()
        .await?
        .into_iter()
        .map(|p| (p.id, p))
        .collect();
    let property_details = ids
        .iter()
        .map(|id| property_details_map.remove(id).unwrap())
        .collect();
    Ok(property_details)
}
