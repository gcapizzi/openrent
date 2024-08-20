use axum::{extract::Multipart, http::StatusCode, response::Result, routing::post, Json, Router};
use serde::Serialize;
use tower_http::services::ServeDir;

mod geo;
mod openrent;

#[derive(Serialize)]
struct Response {
    pub properties: Vec<openrent::Property>,
    pub polygons: Vec<geo::Polygon>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/search", post(search))
        .fallback_service(ServeDir::new("static"));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn search(mut multipart: Multipart) -> Result<Json<Response>> {
    if let Some(field) = multipart.next_field().await? {
        let data = field.text().await?;
        let area = geo::Area::from_kml(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let center = area.center();
        let properties = openrent::get_properties(center.longitude, center.latitude, area.radius())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let predicates: Vec<Box<dyn Fn(&openrent::Property) -> bool>> = vec![
            Box::new(|p| p.live),
            Box::new(|p| area.contains(p.longitude, p.latitude)),
        ];
        let mut filtered_properties: Vec<openrent::Property> = properties
            .into_iter()
            .filter(|p| predicates.iter().all(|pr| pr(p)))
            .collect();
        filtered_properties.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());

        Ok(Json(Response {
            properties: filtered_properties,
            polygons: area.polygons(),
        }))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR.into())
    }
}
