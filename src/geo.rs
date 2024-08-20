use anyhow::{anyhow, Result};
use geo::{BoundingRect, Contains, Geometry, HaversineDistance};
use serde::{Serialize, Serializer};

pub struct Area {
    collection: geo::GeometryCollection<f64>,
    bounding_rectangle: geo::Rect<f64>,
}

#[derive(Debug, Serialize)]
pub struct Polygon {
    pub external: Vec<Point>,
    pub internals: Vec<Vec<Point>>,
}

#[derive(Debug)]
pub struct Point {
    pub longitude: f64,
    pub latitude: f64,
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (&self.latitude, &self.longitude).serialize(serializer)
    }
}

impl Area {
    pub fn from_kml_file<P: AsRef<std::path::Path>>(path: P) -> Result<Area> {
        let mut kml_reader = kml::KmlReader::<_, f64>::from_path(&path)?;
        Ok(Area::from_kml_reader(&mut kml_reader)?)
    }

    pub fn from_kml(string: String) -> Result<Area> {
        let mut kml_reader = kml::KmlReader::<_, f64>::from_string(&string);
        Ok(Area::from_kml_reader(&mut kml_reader)?)
    }

    fn from_kml_reader<B: std::io::BufRead>(
        kml_reader: &mut kml::KmlReader<B, f64>,
    ) -> Result<Area> {
        let kml_data = kml_reader.read()?;
        let collection = kml::quick_collection(kml_data)?;
        let bounding_rectangle = collection
            .bounding_rect()
            .ok_or(anyhow!("no bounding rect"))?;

        Ok(Area {
            collection,
            bounding_rectangle,
        })
    }

    pub fn center(&self) -> Point {
        let coord = self.bounding_rectangle.center();
        Point {
            longitude: coord.x,
            latitude: coord.y,
        }
    }

    pub fn radius(&self) -> u32 {
        let center = geo::Point::from(self.bounding_rectangle.center());
        let max = geo::Point::from(self.bounding_rectangle.max());
        (center.haversine_distance(&max) / 1000.0).round() as u32
    }

    pub fn contains(&self, longitude: f64, latitude: f64) -> bool {
        self.collection.contains(&geo::Coord {
            x: longitude,
            y: latitude,
        })
    }

    pub fn polygons(&self) -> Vec<Polygon> {
        self.collection
            .iter()
            .filter_map(|geometry| {
                if let Geometry::Polygon(p) = geometry {
                    Some(Polygon {
                        external: linestring_to_points(p.exterior()),
                        internals: p.interiors().iter().map(linestring_to_points).collect(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

fn linestring_to_points(l: &geo::LineString) -> Vec<Point> {
    l.coords()
        .map(|c| Point {
            longitude: c.x,
            latitude: c.y,
        })
        .collect()
}
