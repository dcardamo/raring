use super::*;
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Write as _},
    net::TcpListener,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

// A minimal Mapbox Vector Tile encoder for synthetic tiles.
fn varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let byte = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}
fn bytes_field(out: &mut Vec<u8>, field: u64, bytes: &[u8]) {
    varint(out, field << 3 | 2);
    varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}
fn varint_field(out: &mut Vec<u8>, field: u64, v: u64) {
    varint(out, field << 3);
    varint(out, v);
}
fn packed(values: &[u32]) -> Vec<u8> {
    let mut out = vec![];
    for v in values {
        varint(&mut out, u64::from(*v));
    }
    out
}
fn zigzag(v: i64) -> u32 {
    ((v << 1) ^ (v >> 63)) as u32
}
fn geometry(paths: &[Vec<(i64, i64)>], polygon: bool) -> Vec<u32> {
    let (mut g, mut at) = (vec![], (0, 0));
    for path in paths {
        let mut step = |g: &mut Vec<u32>, p: (i64, i64)| {
            g.extend([zigzag(p.0 - at.0), zigzag(p.1 - at.1)]);
            at = p;
        };
        g.push(1 | 1 << 3);
        step(&mut g, path[0]);
        g.push(2 | ((path.len() as u32 - 1) << 3));
        for p in &path[1..] {
            step(&mut g, *p);
        }
        if polygon {
            g.push(7 | 1 << 3);
        }
    }
    g
}
struct Shape {
    kind: u64,
    tags: &'static [(&'static str, &'static str)],
    paths: Vec<Vec<(i64, i64)>>,
}
fn layer(name: &str, shapes: &[Shape]) -> Vec<u8> {
    let (mut keys, mut values) = (Vec::<&str>::new(), Vec::<&str>::new());
    let index = |list: &mut Vec<&'static str>, item: &'static str| {
        list.iter().position(|x| *x == item).unwrap_or_else(|| {
            list.push(item);
            list.len() - 1
        }) as u32
    };
    let mut out = vec![];
    varint_field(&mut out, 15, 2);
    bytes_field(&mut out, 1, name.as_bytes());
    for shape in shapes {
        let tags: Vec<u32> = shape.tags.iter().flat_map(|(k, v)| [index(&mut keys, k), index(&mut values, v)]).collect();
        let mut feature = vec![];
        bytes_field(&mut feature, 2, &packed(&tags));
        varint_field(&mut feature, 3, shape.kind);
        bytes_field(&mut feature, 4, &packed(&geometry(&shape.paths, shape.kind == mvt::POLYGON)));
        bytes_field(&mut out, 2, &feature);
    }
    for key in keys {
        bytes_field(&mut out, 3, key.as_bytes());
    }
    for value in values {
        let mut v = vec![];
        bytes_field(&mut v, 1, value.as_bytes());
        bytes_field(&mut out, 4, &v);
    }
    varint_field(&mut out, 5, 4096);
    out
}
fn square(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<(i64, i64)> {
    vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}
fn shape(kind: u64, tags: &'static [(&'static str, &'static str)], paths: Vec<Vec<(i64, i64)>>) -> Shape {
    Shape { kind, tags, paths }
}
/// Water on the left half, grass, a park, a stream, a primary road and a path
/// near the top, where `loop_route`'s card is, and things the card never
/// draws: a pool, farmland, a road tunnel, rail and buildings.
fn synthetic_tile() -> Vec<u8> {
    let (line, polygon) = (mvt::LINE, mvt::POLYGON);
    let layers = [
        layer("water", &[shape(polygon, &[("class", "lake")], vec![square(-64, -64, 2048, 4160)]), shape(polygon, &[("class", "swimming_pool")], vec![square(3000, 3000, 3100, 3100)])]),
        layer("landcover", &[shape(polygon, &[("class", "grass")], vec![square(2200, 200, 3000, 900)]), shape(polygon, &[("class", "farmland")], vec![square(3200, 200, 3900, 900)])]),
        layer("park", &[shape(polygon, &[("class", "nature_reserve")], vec![square(3100, 300, 3400, 1300)])]),
        layer("waterway", &[shape(line, &[("class", "stream")], vec![vec![(2048, 1000), (4096, 1200)]])]),
        layer("transportation", &[
            shape(line, &[("class", "primary")], vec![vec![(-64, 800), (4160, 800)]]),
            shape(line, &[("class", "path"), ("subclass", "footway")], vec![vec![(2100, 0), (2100, 4096)]]),
            shape(line, &[("class", "minor"), ("brunnel", "tunnel")], vec![vec![(0, 3000), (4096, 3000)]]),
            shape(line, &[("class", "rail")], vec![vec![(0, 3500), (4096, 3500)]]),
        ]),
        layer("building", &[shape(polygon, &[], vec![square(3300, 3300, 3400, 3400)])]),
    ];
    let mut out = vec![];
    for l in layers {
        bytes_field(&mut out, 3, &l);
    }
    out
}

/// A synthetic loop about 1 km across.
fn loop_route() -> Vec<[f64; 2]> {
    (0..=40)
        .map(|i| {
            let a = f64::from(i) / 40. * std::f64::consts::TAU;
            [45.4215 + 0.004 * a.sin(), -75.6972 + 0.006 * a.cos()]
        })
        .collect()
}

fn store_with_workout(id: &str, route: &[[f64; 2]]) -> Store {
    let store = Store::open(":memory:").unwrap();
    put_workout(&store, id, route);
    store
}
fn put_workout(store: &Store, id: &str, route: &[[f64; 2]]) {
    store
        .put("activity", id, "synthetic", &json!({"activityId": id, "activityName": "Loop", "activityTypeDTO": {"typeKey": "running"}, "summaryDTO": {"startTimeLocal": "2025-10-16T06:12:00", "duration": 600, "distance": 3000}}))
        .unwrap();
    let points: Vec<serde_json::Value> = route
        .iter()
        .enumerate()
        .map(|(i, [lat, lon])| json!({"metrics": [i as f64 * 15., 140., lat, lon]}))
        .collect();
    store
        .put("details", id, "synthetic", &json!({"metricDescriptors": [{"key": "directTimestamp"}, {"key": "directHeartRate"}, {"key": "directLatitude"}, {"key": "directLongitude"}], "activityDetailMetrics": points}))
        .unwrap();
}

/// Card-space numbers in a layer's path data.
fn numbers(path: &str) -> Vec<f64> {
    path.split(|c: char| c.is_ascii_alphabetic() || c == ' ').filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect()
}

#[test]
fn fit_matches_the_route_card() {
    let frame = Frame::fit(&loop_route()).unwrap();
    let points: Vec<(f64, f64)> = loop_route()
        .iter()
        .filter_map(|p| mercator(p[0], p[1]))
        .map(|(x, y)| ((x - frame.x0) * frame.scale, (y - frame.y0) * frame.scale))
        .collect();
    let (xs, ys): (Vec<f64>, Vec<f64>) = points.into_iter().unzip();
    let span = |v: &[f64]| v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min);
    let middle = |v: &[f64]| (v.iter().cloned().fold(f64::MIN, f64::max) + v.iter().cloned().fold(f64::MAX, f64::min)) / 2.;
    assert!(span(&xs) <= 310. + 1e-9 && span(&ys) <= 128. + 1e-9);
    assert!((span(&xs) - 310.).abs() < 1e-6 || (span(&ys) - 128.).abs() < 1e-6, "the route fills one side of the fit");
    assert!((middle(&xs) - 179.).abs() < 1e-6 && (middle(&ys) - 88.).abs() < 1e-6);
    assert_eq!(frame.zoom, 13, "a 1 km loop draws from z13 tiles");
    // A long ride draws from coarser tiles; a standing start zooms in only so far.
    let ride = Frame::fit(&[[45.0, -75.9], [45.6, -75.0]]).unwrap();
    assert_eq!(ride.zoom, 9);
    let still = Frame::fit(&[[45.4, -75.7], [45.4, -75.7]]).unwrap();
    assert_eq!((still.scale, still.zoom), (MAX_SCALE, 13));
    assert_eq!(Frame::fit(&[[45.4, -75.7]]), None);
    assert_eq!(Frame::fit(&[[89.0, -75.7], [f64::NAN, 0.]]), None, "points off the map are skipped");
}

#[test]
fn an_area_is_an_eight_by_eight_block_around_the_card() {
    let frame = Frame::fit(&loop_route()).unwrap();
    let tiles = frame.tiles();
    assert!(!tiles.is_empty() && tiles.len() <= 4);
    let areas = frame.areas();
    for tile in &tiles {
        assert!(areas.contains(&area_of(*tile)));
    }
    for area in areas {
        let block = area_tiles(area);
        assert_eq!(block.len(), 64);
        assert!(block.iter().all(|t| area_of(*t) == area));
    }
    // Below z3 the whole world is one block.
    assert_eq!(area_tiles((1, 0, 0)), vec![(1, 0, 0), (1, 0, 1), (1, 1, 0), (1, 1, 1)]);
    assert_eq!(area_of((1, 1, 1)), (1, 0, 0));
}

#[test]
fn decodes_tiles_and_skips_what_the_card_never_draws() {
    let data = synthetic_tile();
    let mut drawn = vec![];
    for layer in mvt::layers(&data) {
        for feature in layer.features() {
            drawn.push((layer.name.to_owned(), kind(&layer, &feature), feature.paths().len()));
        }
    }
    let kinds: Vec<Option<MapLayerKind>> = drawn.iter().map(|d| d.1).collect();
    use MapLayerKind::*;
    assert_eq!(kinds, vec![Some(Water), None, Some(Green), None, Some(Green), Some(Waterway), Some(Major), Some(Path), None, None, None]);
    // Polygon rings come back closed.
    let water = mvt::layers(&data).into_iter().find(|l| l.name == "water").unwrap();
    let ring = &water.features().next().unwrap().paths()[0];
    assert_eq!(ring.first(), ring.last());
    assert_eq!(ring.len(), 5);
    // Truncated or garbage input is never a panic.
    for cut in 0..data.len() {
        for layer in mvt::layers(&data[..cut]) {
            for feature in layer.features() {
                let _ = feature.paths();
            }
        }
    }
    assert!(mvt::layers(&[0xff; 32]).is_empty());
}

#[test]
fn clipping_keeps_only_the_card() {
    let ring = clip_ring(&[(-100., -100.), (100., -100.), (100., 100.), (-100., 100.), (-100., -100.)]);
    assert!(ring.iter().all(|&(x, y)| x >= -MARGIN && y >= -MARGIN && x <= 100. && y <= 100.));
    assert!((area(&[ring.clone(), vec![ring[0]]].concat()).abs() - 104. * 104.).abs() < 1e-6);
    // A line leaving and re-entering the card is two runs; one inside stays whole.
    let runs = clip_line(&[(10., 10.), (100., 10.), (624., 10.), (624., 50.), (100., 50.), (50., 50.)]);
    assert_eq!(runs, vec![vec![(10., 10.), (100., 10.), (CLIP.2, 10.)], vec![(CLIP.2, 50.), (100., 50.), (50., 50.)]]);
    assert!(clip_line(&[(-50., -50.), (-10., -60.)]).is_empty());
}

#[test]
fn simplifying_a_closed_ring_keeps_its_shape() {
    let mut ring: Vec<(f64, f64)> = (0..=64)
        .map(|i| {
            let a = f64::from(i) / 64. * std::f64::consts::TAU;
            (50. + 40. * a.cos(), 50. + 40. * a.sin())
        })
        .collect();
    *ring.last_mut().unwrap() = ring[0];
    let simple = simplify(&ring);
    assert!(simple.len() > 8 && simple.len() < ring.len());
    assert!((area(&simple).abs() - area(&ring).abs()).abs() / area(&ring).abs() < 0.02);
}

fn complete_area(store: &Store, id: &str, frame: &Frame, tile: &[u8]) {
    for area in frame.areas() {
        let tiles: Vec<(Tile, Vec<u8>)> = area_tiles(area).into_iter().map(|t| (t, tile.to_vec())).collect();
        assert!(store.put_map_area(id, area, "https://tiles.example/{z}/{x}/{y}", "now", &tiles).unwrap());
    }
}

#[test]
fn draws_layers_in_order_inside_the_card() {
    let store = store_with_workout("1", &loop_route());
    let frame = Frame::fit(&loop_route()).unwrap();
    assert_eq!(draw(&store, &frame), None, "nothing to draw before the area is stored");
    complete_area(&store, "1", &frame, &synthetic_tile());
    let layers = draw(&store, &frame).unwrap();
    use MapLayerKind::*;
    assert_eq!(layers.iter().map(|l| l.kind).collect::<Vec<_>>(), vec![Green, Water, Waterway, Path, Major]);
    for layer in &layers {
        let n = numbers(&layer.path);
        assert!(n.chunks(2).all(|p| p[0] >= CLIP.0 && p[0] <= CLIP.2 && p[1] >= CLIP.1 && p[1] <= CLIP.3), "{:?} stays in the card", layer.kind);
        assert!(n.iter().all(|v| (v * 10.).round() == v * 10.), "one decimal");
        assert_eq!(layer.path.ends_with('Z'), is_area(layer.kind));
    }
}

#[test]
fn reading_a_workout_never_draws_and_follows_the_setting() {
    let route = loop_route();
    let store = store_with_workout("1", &route);
    let view = crate::screens::workout(&store, "1").unwrap();
    let map = view.map.unwrap();
    assert_eq!(map.state, RouteMapState::Missing);
    assert!(map.layers.is_empty());
    assert_eq!(map.frame, Frame::fit(&view.route).unwrap().view());
    // Tiles alone don't make a card; only the explicit fetch draws one.
    complete_area(&store, "1", &Frame::fit(&view.route).unwrap(), &synthetic_tile());
    assert_eq!(stored(&store, "1", &view.route).unwrap().state, RouteMapState::Missing);
    let off = crate::view::Settings { street_maps: false, ..Default::default() };
    store.set_settings(&off).unwrap();
    assert_eq!(crate::screens::workout(&store, "1").unwrap().map, None);
    // No GPS, no map.
    put_workout(&store, "2", &[]);
    store.set_settings(&Default::default()).unwrap();
    assert_eq!(crate::screens::workout(&store, "2").unwrap().map, None);
}

/// A tile server on localhost: `handle` answers each path with a status and
/// body. Counts requests, keeps their heads, and records the most answered at
/// once.
struct Server {
    url: String,
    heads: Arc<Mutex<Vec<String>>>,
    busiest: Arc<AtomicUsize>,
}
fn serve(handle: impl Fn(&str, &str) -> (u16, Vec<u8>) + Send + Sync + 'static) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (heads, busiest, active) = (Arc::new(Mutex::new(vec![])), Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (h, b, handle) = (Arc::clone(&heads), Arc::clone(&busiest), Arc::new(handle));
    let base = url.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let (h, b, a, handle, base) = (Arc::clone(&h), Arc::clone(&b), Arc::clone(&active), Arc::clone(&handle), base.clone());
            std::thread::spawn(move || {
                let mut head = String::new();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                    head.push_str(&line);
                }
                let path = head.split_whitespace().nth(1).unwrap_or("").to_owned();
                let now = a.fetch_add(1, Ordering::SeqCst) + 1;
                b.fetch_max(now, Ordering::SeqCst);
                h.lock().unwrap().push(head);
                std::thread::sleep(std::time::Duration::from_millis(15));
                let (status, body) = handle(&base, &path);
                // Counted down before answering, so a request the client sends in its place can't overlap it.
                a.fetch_sub(1, Ordering::SeqCst);
                let _ = write!(stream, "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                let _ = stream.write_all(&body);
            });
        }
    });
    Server { url, heads, busiest }
}
fn source(server: &Server) -> Source {
    Source { tilejson: format!("{}/planet", server.url), host: format!("{}/tiles/", server.url) }
}
fn tile_server(failing: Option<&'static str>) -> Server {
    let tile = synthetic_tile();
    serve(move |base, path| match path {
        "/planet" => (200, format!(r#"{{"tiles":["{base}/tiles/{{z}}/{{x}}/{{y}}.pbf"],"maxzoom":14}}"#).into_bytes()),
        p if Some(p) == failing => (500, vec![]),
        p if p.starts_with("/tiles/13/") && p.ends_with("0.pbf") => (404, vec![]),
        p if p.starts_with("/tiles/") => (200, tile.clone()),
        _ => (404, vec![]),
    })
}
fn counts(store: &Store) -> (i64, i64, i64) {
    store.map_rows()
}

#[tokio::test(flavor = "multi_thread")]
async fn downloads_an_area_once_and_draws_each_workout_once() {
    let route = loop_route();
    let store = store_with_workout("1", &route);
    let server = tile_server(None);
    let areas = Frame::fit(&route).unwrap().areas().len() as i64;
    let map = fetch(&store, "1", &route, &source(&server), "Raring/9.9.9 (+https://raring.app)").await.unwrap().unwrap();
    assert_eq!(map.state, RouteMapState::Ready);
    assert!(map.layers.iter().any(|l| l.kind == MapLayerKind::Water));
    let heads = server.heads.lock().unwrap().clone();
    assert_eq!(heads.len() as i64, 1 + 64 * areas, "the TileJSON, then every tile of each area");
    assert!(heads.iter().all(|h| h.to_ascii_lowercase().contains("user-agent: raring/9.9.9 (+https://raring.app)")));
    let busiest = server.busiest.load(Ordering::SeqCst);
    assert!((2..=PARALLEL).contains(&busiest), "parallel, never more than OpenFreeMap allows: {busiest}");
    assert_eq!(counts(&store), (areas, 64 * areas, 1), "empty tiles are kept too");
    // The page read now shows the drawn card.
    assert_eq!(crate::screens::workout(&store, "1").unwrap().map, Some(map.clone()));
    // Opening it again, or another workout in the same area, makes no request.
    assert_eq!(fetch(&store, "1", &route, &source(&server), "Raring").await.unwrap(), Some(map));
    let nearby: Vec<[f64; 2]> = route.iter().map(|[lat, lon]| [lat + 0.0005, lon + 0.0005]).collect();
    put_workout(&store, "2", &nearby);
    if Frame::fit(&nearby).unwrap().areas().iter().all(|a| store.map_area_stored(*a)) {
        assert!(fetch(&store, "2", &nearby, &source(&server), "Raring").await.unwrap().is_some());
        assert_eq!(server.heads.lock().unwrap().len(), heads.len());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_failed_tile_stores_nothing() {
    let route = loop_route();
    let store = store_with_workout("1", &route);
    let (z, x, y) = area_tiles(Frame::fit(&route).unwrap().areas()[0])[5];
    let failing: &'static str = Box::leak(format!("/tiles/{z}/{x}/{y}.pbf").into_boxed_str());
    let server = tile_server(Some(failing));
    assert!(fetch(&store, "1", &route, &source(&server), "Raring").await.is_err());
    assert_eq!(counts(&store), (0, 0, 0));
    assert_eq!(crate::screens::workout(&store, "1").unwrap().map.unwrap().state, RouteMapState::Missing);
}

#[tokio::test(flavor = "multi_thread")]
async fn tiles_must_stay_on_the_sources_host() {
    let route = loop_route();
    let store = store_with_workout("1", &route);
    let server = serve(|_, path| match path {
        "/planet" => (200, br#"{"tiles":["https://elsewhere.example/{z}/{x}/{y}.pbf"]}"#.to_vec()),
        _ => (200, vec![]),
    });
    assert!(fetch(&store, "1", &route, &source(&server), "Raring").await.is_err());
    assert_eq!(server.heads.lock().unwrap().len(), 1);
    assert_eq!(counts(&store), (0, 0, 0));
}

#[tokio::test(flavor = "multi_thread")]
async fn nothing_is_requested_or_kept_with_street_maps_off_or_the_workout_gone() {
    let route = loop_route();
    let store = store_with_workout("1", &route);
    let server = tile_server(None);
    store.set_settings(&crate::view::Settings { street_maps: false, ..Default::default() }).unwrap();
    assert_eq!(fetch(&store, "1", &route, &source(&server), "Raring").await.unwrap(), None);
    assert!(server.heads.lock().unwrap().is_empty());
    // A workout erased while its area downloads (sign-out, another account) keeps nothing.
    store.set_settings(&Default::default()).unwrap();
    store.forget_account().unwrap();
    assert_eq!(fetch(&store, "1", &route, &source(&server), "Raring").await.unwrap(), None);
    assert_eq!(counts(&store), (0, 0, 0));
}

#[test]
fn turning_street_maps_off_or_signing_out_erases_them() {
    let raring = crate::Raring::open(":memory:").unwrap();
    let route = loop_route();
    put_workout(&raring.store, "1", &route);
    let frame = Frame::fit(&route).unwrap();
    complete_area(&raring.store, "1", &frame, &synthetic_tile());
    assert!(raring.store.put_route_map("1", "{}").unwrap());
    let patch: crate::settings::SettingsPatch = serde_json::from_value(json!({"streetMaps": false})).unwrap();
    raring.patch_settings_for_account(patch, None).unwrap();
    assert_eq!(counts(&raring.store), (0, 0, 0));
    assert!(!raring.store.put_route_map("1", "{}").unwrap(), "refused while off");
    let patch: crate::settings::SettingsPatch = serde_json::from_value(json!({"streetMaps": true})).unwrap();
    raring.patch_settings_for_account(patch, None).unwrap();
    complete_area(&raring.store, "1", &frame, &synthetic_tile());
    assert!(raring.store.put_route_map("1", "{}").unwrap());
    raring.forget_account().unwrap();
    assert_eq!(counts(&raring.store), (0, 0, 0));
}
