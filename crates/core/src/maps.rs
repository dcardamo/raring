//! Street maps under workout routes (Rules: Street maps). OpenStreetMap vector
//! tiles come from OpenFreeMap an area at a time and stay on the phone; each
//! route card is drawn once into SVG path data and kept with its workout.
mod mvt;
#[cfg(test)]
mod tests;

use crate::{
    store::Store,
    view::{MapFrame, MapLayer, MapLayerKind, RouteMapState, RouteMapView},
    Error, Result,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Write, io::Read};

/// RouteMap.svelte's card: a 358×176 viewBox with the route fitted into at
/// most 310×128 around its centre. The UI repeats this fit for cards without
/// a map, so a map arriving never moves the route.
const CARD: (f64, f64) = (358., 176.);
const FIT: (f64, f64) = (310., 128.);
/// The closest a card zooms in, about 850 m across at the equator, so a short
/// route still has streets around it.
const MAX_SCALE: f64 = 256. * 65536.;
/// The tile zoom puts about this many card pixels across each tile.
const TILE_PX: f64 = 140.;
/// z13 has every road, path and water class the card draws.
const MAX_ZOOM: u8 = 13;
/// An area is an 8×8 block of tiles at the card's zoom, several times the
/// card's size, so the tile server never sees where the route is within it.
const AREA_SHIFT: u8 = 3;
/// OpenFreeMap allows offline downloads of at most 3,000 tiles or 50 MB, with
/// at most 4 parallel requests and a User-Agent naming the app.
const AREA_BYTES: usize = 50_000_000;
const PARALLEL: usize = 4;
const TILE_BYTES: usize = 8_000_000;
const TILEJSON_BYTES: usize = 1_000_000;
/// Line ends and widths reach a little past the card's edge.
const MARGIN: f64 = 4.;
/// Smaller differences than this, in card pixels, are dropped.
const TOLERANCE: f64 = 0.35;
/// Bump when drawing changes, so cards drawn before are drawn again.
const DRAWING: u32 = 1;

/// One download at a time across the app: a card opened twice, or two cards in
/// one area, must not fetch an area twice or exceed OpenFreeMap's 4 requests.
static DOWNLOADS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// A TileJSON whose tile URLs must stay on `host`.
pub struct Source {
    pub tilejson: String,
    pub host: String,
}

impl Source {
    pub fn openfreemap() -> Self {
        Self {
            tilejson: "https://tiles.openfreemap.org/planet".into(),
            host: "https://tiles.openfreemap.org/".into(),
        }
    }
}

pub fn user_agent(app_version: &str) -> String {
    format!("Raring/{app_version} (+https://raring.app)")
}

type Tile = (u8, u32, u32);
/// The tile zoom and the block's x and y: a tile's x and y shifted by
/// `AREA_SHIFT`, or by the zoom itself below z3.
type Area = (u8, u32, u32);

fn area_of((z, x, y): Tile) -> Area {
    let shift = AREA_SHIFT.min(z);
    (z, x >> shift, y >> shift)
}

fn area_tiles((z, x, y): Area) -> Vec<Tile> {
    let shift = AREA_SHIFT.min(z);
    let side = 1u32 << shift;
    (0..side)
        .flat_map(|dx| (0..side).map(move |dy| (z, (x << shift) + dx, (y << shift) + dy)))
        .collect()
}

/// Web Mercator, 0–1 across the world with y down.
fn mercator(lat: f64, lon: f64) -> Option<(f64, f64)> {
    if !lat.is_finite() || !lon.is_finite() || lat.abs() > 85.051_128_78 || lon.abs() > 180. {
        return None;
    }
    Some(((lon + 180.) / 360., (1. - lat.to_radians().tan().asinh() / std::f64::consts::PI) / 2.))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Frame {
    x0: f64,
    y0: f64,
    scale: f64,
    zoom: u8,
}

impl Frame {
    fn fit(route: &[[f64; 2]]) -> Option<Self> {
        let points: Vec<(f64, f64)> = route.iter().filter_map(|p| mercator(p[0], p[1])).collect();
        if points.len() < 2 {
            return None;
        }
        let (mut x_min, mut x_max, mut y_min, mut y_max) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for &(x, y) in &points {
            (x_min, x_max, y_min, y_max) = (x_min.min(x), x_max.max(x), y_min.min(y), y_max.max(y));
        }
        let room = |span: f64, size: f64| if span > 0. { size / span } else { f64::INFINITY };
        let scale = room(x_max - x_min, FIT.0).min(room(y_max - y_min, FIT.1)).min(MAX_SCALE);
        Some(Self {
            x0: (x_min + x_max) / 2. - CARD.0 / 2. / scale,
            y0: (y_min + y_max) / 2. - CARD.1 / 2. / scale,
            scale,
            zoom: (scale / TILE_PX).log2().round().clamp(0., f64::from(MAX_ZOOM)) as u8,
        })
    }
    fn view(&self) -> MapFrame {
        MapFrame { x0: self.x0, y0: self.y0, scale: self.scale }
    }
    /// The tiles at the card's zoom that reach into the card.
    fn tiles(&self) -> Vec<Tile> {
        let size = f64::from(1u32 << self.zoom);
        let margin = MARGIN / self.scale;
        let range = |from: f64, span: f64| {
            let tile = |v: f64| (v * size).floor().clamp(0., size - 1.) as u32;
            tile(from - margin)..=tile(from + span / self.scale + margin)
        };
        let (xs, ys) = (range(self.x0, CARD.0), range(self.y0, CARD.1));
        xs.flat_map(|x| ys.clone().map(move |y| (self.zoom, x, y))).collect()
    }
    fn areas(&self) -> Vec<Area> {
        let mut areas: Vec<Area> = self.tiles().into_iter().map(area_of).collect();
        areas.sort_unstable();
        areas.dedup();
        areas
    }
}

#[derive(Serialize, Deserialize)]
struct Drawn {
    drawing: u32,
    frame: MapFrame,
    layers: Vec<MapLayer>,
}

fn drawn(store: &Store, id: &str, frame: &Frame) -> Option<RouteMapView> {
    let drawn: Drawn = serde_json::from_str(&store.route_map(id)?).ok()?;
    (drawn.drawing == DRAWING && drawn.frame == frame.view()).then_some(RouteMapView {
        frame: drawn.frame,
        state: RouteMapState::Ready,
        layers: drawn.layers,
    })
}

/// A workout's drawn card, or `Missing` when it still has to be drawn. Never
/// downloads or draws, so opening a page stays a local read.
pub fn stored(store: &Store, id: &str, route: &[[f64; 2]]) -> Option<RouteMapView> {
    if !store.settings().street_maps {
        return None;
    }
    let frame = Frame::fit(route)?;
    Some(drawn(store, id, &frame).unwrap_or(RouteMapView {
        frame: frame.view(),
        state: RouteMapState::Missing,
        layers: vec![],
    }))
}

/// Downloads the areas a workout's card needs that aren't on the phone, then
/// draws the card and keeps it. `None` with street maps off, without a route,
/// or when the workout was erased meanwhile.
pub async fn fetch(
    store: &Store,
    id: &str,
    route: &[[f64; 2]],
    source: &Source,
    user_agent: &str,
) -> Result<Option<RouteMapView>> {
    if !store.settings().street_maps {
        return Ok(None);
    }
    let Some(frame) = Frame::fit(route) else { return Ok(None) };
    if let Some(view) = drawn(store, id, &frame) {
        return Ok(Some(view));
    }
    let missing = |store: &Store| -> Vec<Area> { frame.areas().into_iter().filter(|a| !store.map_area_stored(*a)).collect() };
    if !missing(store).is_empty() {
        let _one_at_a_time = DOWNLOADS.lock().await;
        // Another card may have stored them while this one waited.
        let missing = missing(store);
        if !missing.is_empty() {
            if !store.maps_allowed(id) {
                return Ok(None);
            }
            let host = source.host.clone();
            let http = reqwest::Client::builder()
                .user_agent(user_agent)
                .timeout(std::time::Duration::from_secs(30))
                // Redirects may not leave the source's host.
                .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                    if attempt.previous().len() < 5 && attempt.url().as_str().starts_with(&host) {
                        attempt.follow()
                    } else {
                        attempt.stop()
                    }
                }))
                .build()
                .map_err(|_| Error::Map("no network client"))?;
            let template = tile_template(&http, source).await?;
            for area in missing {
                let tiles = match download(store, id, &http, &template, area).await {
                    Ok(tiles) => tiles,
                    // Turned off or erased meanwhile: no map rather than a failure.
                    Err(_) if !store.maps_allowed(id) => return Ok(None),
                    Err(e) => return Err(e),
                };
                let at = chrono::Utc::now().to_rfc3339();
                if !store.put_map_area(id, area, &template, &at, &tiles)? {
                    return Ok(None);
                }
            }
        }
    }
    let layers = draw(store, &frame).ok_or(Error::Map("map data missing"))?;
    let drawn = Drawn { drawing: DRAWING, frame: frame.view(), layers };
    let json = serde_json::to_string(&drawn).map_err(|_| Error::Map("drawing"))?;
    if !store.put_route_map(id, &json)? {
        return Ok(None);
    }
    Ok(Some(RouteMapView { frame: drawn.frame, state: RouteMapState::Ready, layers: drawn.layers }))
}

async fn body(mut response: reqwest::Response, limit: usize) -> Result<Vec<u8>> {
    let mut body = vec![];
    while let Some(chunk) = response.chunk().await.map_err(|_| Error::Map("network"))? {
        body.extend_from_slice(&chunk);
        if body.len() > limit {
            return Err(Error::Map("response too large"));
        }
    }
    Ok(body)
}

/// The tile URL template the TileJSON names, refused unless it stays on the
/// source's host.
async fn tile_template(http: &reqwest::Client, source: &Source) -> Result<String> {
    let response = http.get(&source.tilejson).send().await.map_err(|_| Error::Map("network"))?;
    if !response.status().is_success() {
        return Err(Error::Map("tile source unavailable"));
    }
    let json: serde_json::Value =
        serde_json::from_slice(&body(response, TILEJSON_BYTES).await?).map_err(|_| Error::Map("tile source"))?;
    json["tiles"][0]
        .as_str()
        .filter(|t| t.starts_with(&source.host) && ["{z}", "{x}", "{y}"].iter().all(|p| t.contains(p)))
        .map(str::to_owned)
        .ok_or(Error::Map("tile source"))
}

/// Every tile of an area, or an error and nothing. No tile is requested after
/// street maps are turned off or the workout is erased.
async fn download(
    store: &Store,
    id: &str,
    http: &reqwest::Client,
    template: &str,
    area: Area,
) -> Result<Vec<(Tile, Vec<u8>)>> {
    let mut requests = futures_util::stream::iter(area_tiles(area))
        .map(|tile| async move {
            if !store.maps_allowed(id) {
                return Err(Error::Map("street maps are off"));
            }
            tile_bytes(http, template, tile).await.map(|bytes| (tile, bytes))
        })
        .buffer_unordered(PARALLEL);
    let (mut tiles, mut total) = (vec![], 0);
    while let Some(tile) = requests.next().await {
        let tile = tile?;
        total += tile.1.len();
        if total > AREA_BYTES {
            return Err(Error::Map("area too large"));
        }
        tiles.push(tile);
    }
    Ok(tiles)
}

/// A tile's vector data; empty where the source has none.
async fn tile_bytes(http: &reqwest::Client, template: &str, (z, x, y): Tile) -> Result<Vec<u8>> {
    let url = template
        .replace("{z}", &z.to_string())
        .replace("{x}", &x.to_string())
        .replace("{y}", &y.to_string());
    let response = http.get(url).send().await.map_err(|_| Error::Map("network"))?;
    match response.status().as_u16() {
        200 => {}
        204 | 404 => return Ok(vec![]),
        _ => return Err(Error::Map("tile unavailable")),
    }
    let bytes = body(response, TILE_BYTES).await?;
    // Tiles stored gzipped are sometimes served without saying so.
    if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut out = vec![];
        flate2::read::GzDecoder::new(bytes.as_slice())
            .take(TILE_BYTES as u64 + 1)
            .read_to_end(&mut out)
            .map_err(|_| Error::Map("tile"))?;
        if out.len() > TILE_BYTES {
            return Err(Error::Map("tile too large"));
        }
        return Ok(out);
    }
    Ok(bytes)
}

/// What a feature draws as, from the OpenMapTiles schema; tunnels, rail,
/// ferries, buildings, labels and unbuilt roads draw nothing.
fn kind(layer: &mvt::Layer, feature: &mvt::Feature) -> Option<MapLayerKind> {
    let class = layer.tag(feature, "class").unwrap_or("");
    if layer.tag(feature, "brunnel") == Some("tunnel") {
        return None;
    }
    match (layer.name, feature.kind) {
        ("water", mvt::POLYGON) if class != "swimming_pool" => Some(MapLayerKind::Water),
        ("waterway", mvt::LINE) => Some(MapLayerKind::Waterway),
        ("park", mvt::POLYGON) => Some(MapLayerKind::Green),
        ("landcover", mvt::POLYGON) if matches!(class, "grass" | "wood") => Some(MapLayerKind::Green),
        ("transportation", mvt::LINE) => match class {
            "motorway" | "trunk" | "primary" => Some(MapLayerKind::Major),
            "secondary" | "tertiary" => Some(MapLayerKind::Road),
            "minor" | "service" | "track" | "raceway" | "busway" => Some(MapLayerKind::Minor),
            "path" | "pier" => Some(MapLayerKind::Path),
            _ => None,
        },
        _ => None,
    }
}

fn is_area(kind: MapLayerKind) -> bool {
    matches!(kind, MapLayerKind::Green | MapLayerKind::Water)
}

/// The card's layers from stored tiles, or `None` while one isn't stored.
fn draw(store: &Store, frame: &Frame) -> Option<Vec<MapLayer>> {
    let mut paths: BTreeMap<MapLayerKind, String> = BTreeMap::new();
    let size = f64::from(1u32 << frame.zoom);
    for (z, x, y) in frame.tiles() {
        let data = store.map_tile(z, x, y)?;
        for layer in mvt::layers(&data) {
            if !matches!(layer.name, "water" | "waterway" | "park" | "landcover" | "transportation") {
                continue;
            }
            let extent = f64::from(layer.extent);
            let card = |(px, py): (i64, i64)| {
                (
                    ((f64::from(x) + px as f64 / extent) / size - frame.x0) * frame.scale,
                    ((f64::from(y) + py as f64 / extent) / size - frame.y0) * frame.scale,
                )
            };
            for feature in layer.features() {
                let Some(kind) = kind(&layer, &feature) else { continue };
                let out = paths.entry(kind).or_default();
                for path in feature.paths() {
                    let points: Vec<(f64, f64)> = path.into_iter().map(card).collect();
                    if is_area(kind) {
                        let mut ring = clip_ring(&points);
                        if let Some(&first) = ring.first() {
                            ring.push(first);
                        }
                        let ring = simplify(&ring);
                        if ring.len() > 3 && area(&ring).abs() >= 0.25 {
                            write_path(out, &ring[..ring.len() - 1], true);
                        }
                    } else {
                        for run in clip_line(&points) {
                            let run = simplify(&run);
                            if !tiny(&run) {
                                write_path(out, &run, false);
                            }
                        }
                    }
                }
            }
        }
    }
    Some(paths.into_iter().filter(|(_, p)| !p.is_empty()).map(|(kind, path)| MapLayer { kind, path }).collect())
}

/// The card, widened by the margin: (left, top, right, bottom).
const CLIP: (f64, f64, f64, f64) = (-MARGIN, -MARGIN, CARD.0 + MARGIN, CARD.1 + MARGIN);

/// Sutherland–Hodgman against the card; the result is an open ring.
fn clip_ring(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut ring = points.to_vec();
    for edge in 0..4 {
        let inside = |p: (f64, f64)| match edge {
            0 => p.0 >= CLIP.0,
            1 => p.1 >= CLIP.1,
            2 => p.0 <= CLIP.2,
            _ => p.1 <= CLIP.3,
        };
        let cross = |a: (f64, f64), b: (f64, f64)| match edge {
            0 | 2 => {
                let x = if edge == 0 { CLIP.0 } else { CLIP.2 };
                (x, a.1 + (x - a.0) / (b.0 - a.0) * (b.1 - a.1))
            }
            _ => {
                let y = if edge == 1 { CLIP.1 } else { CLIP.3 };
                (a.0 + (y - a.1) / (b.1 - a.1) * (b.0 - a.0), y)
            }
        };
        let input = std::mem::take(&mut ring);
        let Some(&last) = input.last() else { break };
        let mut previous = last;
        for &point in &input {
            match (inside(previous), inside(point)) {
                (true, true) => ring.push(point),
                (true, false) => ring.push(cross(previous, point)),
                (false, true) => {
                    ring.push(cross(previous, point));
                    ring.push(point);
                }
                (false, false) => {}
            }
            previous = point;
        }
    }
    ring
}

/// Liang–Barsky on each segment; the parts of a line inside the card.
fn clip_line(points: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    let (mut runs, mut run) = (vec![], Vec::<(f64, f64)>::new());
    for pair in points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let Some((start, end)) = clip_segment(a, b) else {
            if run.len() > 1 {
                runs.push(std::mem::take(&mut run));
            }
            run.clear();
            continue;
        };
        if run.last() != Some(&start) {
            if run.len() > 1 {
                runs.push(std::mem::take(&mut run));
            }
            run.clear();
            run.push(start);
        }
        run.push(end);
        if end != b {
            runs.push(std::mem::take(&mut run));
        }
    }
    if run.len() > 1 {
        runs.push(run);
    }
    runs
}

fn clip_segment(a: (f64, f64), b: (f64, f64)) -> Option<((f64, f64), (f64, f64))> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let (mut t0, mut t1) = (0f64, 1f64);
    for (p, q) in [(-dx, a.0 - CLIP.0), (dx, CLIP.2 - a.0), (-dy, a.1 - CLIP.1), (dy, CLIP.3 - a.1)] {
        if p == 0. {
            if q < 0. {
                return None;
            }
        } else if p < 0. {
            t0 = t0.max(q / p);
        } else {
            t1 = t1.min(q / p);
        }
        if t0 > t1 {
            return None;
        }
    }
    // Unclipped ends stay exact, so consecutive segments join into one line.
    let at = |t: f64| (a.0 + t * dx, a.1 + t * dy);
    Some((if t0 == 0. { a } else { at(t0) }, if t1 == 1. { b } else { at(t1) }))
}

/// Douglas–Peucker, measuring to the segment rather than its line, so a line
/// that doubles back is kept. A closed ring's first segment has no length.
fn simplify(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let last = points.len().saturating_sub(1);
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    (keep[0], keep[last]) = (true, true);
    let mut stack = vec![(0, last)];
    while let Some((i, j)) = stack.pop() {
        let (a, b) = (points[i], points[j]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let length = dx.hypot(dy);
        let (mut far, mut at) = (0., i);
        for (k, p) in points.iter().enumerate().take(j).skip(i + 1) {
            let t = if length > 0. { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / (length * length)).clamp(0., 1.) } else { 0. };
            let d = (p.0 - a.0 - t * dx).hypot(p.1 - a.1 - t * dy);
            if d > far {
                (far, at) = (d, k);
            }
        }
        if far > TOLERANCE {
            keep[at] = true;
            stack.extend([(i, at), (at, j)]);
        }
    }
    points.iter().zip(keep).filter_map(|(p, k)| k.then_some(*p)).collect()
}

fn area(ring: &[(f64, f64)]) -> f64 {
    ring.windows(2).map(|w| w[0].0 * w[1].1 - w[1].0 * w[0].1).sum::<f64>() / 2.
}

fn tiny(line: &[(f64, f64)]) -> bool {
    let (mut x_min, mut x_max, mut y_min, mut y_max) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for &(x, y) in line {
        (x_min, x_max, y_min, y_max) = (x_min.min(x), x_max.max(x), y_min.min(y), y_max.max(y));
    }
    line.len() < 2 || (x_max - x_min < 0.5 && y_max - y_min < 0.5)
}

fn write_path(out: &mut String, points: &[(f64, f64)], close: bool) {
    for (i, &(x, y)) in points.iter().enumerate() {
        out.push(if i == 0 { 'M' } else { 'L' });
        let (x, y) = ((x * 10.).round() / 10. + 0., (y * 10.).round() / 10. + 0.);
        let _ = write!(out, "{x} {y}");
    }
    if close {
        out.push('Z');
    }
}
