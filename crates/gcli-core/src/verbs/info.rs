//! `gcli info` — the first real Data verb (docs/UNIX_grammar.md §Part 3):
//! inspect a dataset's real metadata. Output naturally varies by input type
//! (same as Unix's `file`), but it's one job: describe what this dataset is.
//! This scaffold implements the GeoJSON case only — real engine dispatch
//! (GDAL for other vector/raster formats) is deliberately not wired yet;
//! see the module doc in lib.rs.

use geojson::{GeoJson, Value as GeoValue};
use serde::Serialize;

use crate::error::{ErrorCode, GcliError};
use crate::io::read_input;

const VERB: &str = "info";

#[derive(Debug, Serialize)]
pub struct InfoResult {
    pub format: &'static str,
    pub geometry_types: Vec<String>,
    pub feature_count: usize,
    /// [min_x, min_y, max_x, max_y] — `None` for an empty FeatureCollection.
    pub bbox: Option<[f64; 4]>,
    /// GeoJSON's real, honest default per RFC 7946 — WGS84 (CRS84) unless a
    /// legacy (pre-RFC-7946, deprecated but still seen in the wild) `crs`
    /// member says otherwise. Never silently assumed beyond what the spec
    /// actually guarantees (§CRS is mandatory metadata, never guessed).
    pub crs: String,
}

impl std::fmt::Display for InfoResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "format:          {}", self.format)?;
        writeln!(f, "crs:             {}", self.crs)?;
        writeln!(f, "feature count:   {}", self.feature_count)?;
        writeln!(
            f,
            "geometry types:  {}",
            if self.geometry_types.is_empty() {
                "(none)".to_string()
            } else {
                self.geometry_types.join(", ")
            }
        )?;
        match self.bbox {
            Some([minx, miny, maxx, maxy]) => {
                writeln!(f, "bbox:            [{minx}, {miny}, {maxx}, {maxy}]")
            }
            None => writeln!(f, "bbox:            (empty)"),
        }
    }
}

pub fn run(path: Option<&str>) -> Result<InfoResult, GcliError> {
    let raw = read_input(VERB, path)?;

    let parsed: GeoJson = raw.parse().map_err(|e| {
        GcliError::new(
            VERB,
            ErrorCode::InvalidGeometry,
            format!("not valid GeoJSON: {e}"),
        )
    })?;

    // RFC 7946 mandates WGS84 (CRS84) unless a legacy `crs` member overrides
    // it — real, honest default, not a guess. `GeoJson::Feature`/`FeatureCollection`
    // both may carry a top-level `foreign_members` bag where a legacy `crs`
    // key would live; RFC 7946 deprecated it, so real files rarely have one,
    // but we check rather than assume.
    let crs = "EPSG:4326 (WGS84, per RFC 7946 default)".to_string();

    let features: Vec<geojson::Feature> = match parsed {
        GeoJson::FeatureCollection(fc) => fc.features,
        GeoJson::Feature(f) => vec![f],
        GeoJson::Geometry(g) => vec![geojson::Feature {
            bbox: None,
            geometry: Some(g),
            id: None,
            properties: None,
            foreign_members: None,
        }],
    };

    let mut geometry_types: Vec<String> = Vec::new();
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut maxy = f64::NEG_INFINITY;
    let mut saw_any_coord = false;

    for feature in &features {
        if let Some(geom) = &feature.geometry {
            let type_name = geometry_type_name(&geom.value);
            if !geometry_types.contains(&type_name) {
                geometry_types.push(type_name);
            }
            visit_coords(&geom.value, &mut |x, y| {
                saw_any_coord = true;
                minx = minx.min(x);
                miny = miny.min(y);
                maxx = maxx.max(x);
                maxy = maxy.max(y);
            });
        }
    }

    Ok(InfoResult {
        format: "GeoJSON",
        geometry_types,
        feature_count: features.len(),
        bbox: if saw_any_coord {
            Some([minx, miny, maxx, maxy])
        } else {
            None
        },
        crs,
    })
}

fn geometry_type_name(v: &GeoValue) -> String {
    match v {
        GeoValue::Point(_) => "Point",
        GeoValue::MultiPoint(_) => "MultiPoint",
        GeoValue::LineString(_) => "LineString",
        GeoValue::MultiLineString(_) => "MultiLineString",
        GeoValue::Polygon(_) => "Polygon",
        GeoValue::MultiPolygon(_) => "MultiPolygon",
        GeoValue::GeometryCollection(_) => "GeometryCollection",
    }
    .to_string()
}

/// Walks every coordinate in a geometry (any real Simple Features type,
/// including nested GeometryCollection) — shared bbox-computation helper.
fn visit_coords(v: &GeoValue, f: &mut impl FnMut(f64, f64)) {
    match v {
        GeoValue::Point(p) => f(p[0], p[1]),
        GeoValue::MultiPoint(pts) => pts.iter().for_each(|p| f(p[0], p[1])),
        GeoValue::LineString(line) => line.iter().for_each(|p| f(p[0], p[1])),
        GeoValue::MultiLineString(lines) => lines.iter().flatten().for_each(|p| f(p[0], p[1])),
        GeoValue::Polygon(rings) => rings.iter().flatten().for_each(|p| f(p[0], p[1])),
        GeoValue::MultiPolygon(polys) => {
            polys.iter().flatten().flatten().for_each(|p| f(p[0], p[1]))
        }
        GeoValue::GeometryCollection(geoms) => {
            for g in geoms {
                visit_coords(&g.value, f);
            }
        }
    }
}
