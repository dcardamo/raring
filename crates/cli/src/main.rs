use anyhow::{bail, Context, Result};
use chrono::{Local, TimeZone};
use raring_core::{garmin, Raring, Tokens};
use serde_json::Value;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
fn home() -> PathBuf {
    std::env::var_os("RARING_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(".data")
        })
}
fn token_path() -> PathBuf {
    home().join("garmin_tokens.json")
}
fn read_tokens() -> Result<Tokens> {
    Ok(serde_json::from_str(&fs::read_to_string(token_path())?)?)
}
#[cfg(unix)]
fn save_tokens(tokens: &Tokens) -> Result<()> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let dir = home();
    fs::create_dir_all(&dir)?;
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    let tmp = dir.join(format!(".tokens-{}.tmp", std::process::id()));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&tmp)?;
        serde_json::to_writer(&mut f, tokens)?;
        f.sync_all()?;
        fs::rename(&tmp, token_path())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result
}
#[cfg(not(unix))]
fn save_tokens(tokens: &Tokens) -> Result<()> {
    fs::create_dir_all(home())?;
    fs::write(token_path(), serde_json::to_vec(tokens)?)?;
    Ok(())
}
fn prompt(text: &str) -> Result<String> {
    print!("{text}");
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim().into())
}
fn show<T: serde::Serialize>(v: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}
fn scrub(v: &mut Value) {
    scrub_inner(v, &mut None, &mut None);
}
fn device_ids(list: &Value) -> Vec<String> {
    list.as_array()
        .or_else(|| list.get("devices").and_then(Value::as_array))
        .into_iter()
        .flatten()
        .filter_map(|device| {
            device["deviceId"]
                .as_str()
                .map(str::to_owned)
                .or_else(|| device["deviceId"].as_u64().map(|id| id.to_string()))
        })
        .collect()
}
fn shift(value: &mut Value, origin: &mut Option<f64>) {
    if let Some(n) = value.as_f64() {
        let first = *origin.get_or_insert(n);
        *value = Value::from(((n - first) * 100_000.0).round() / 100_000.0);
    } else {
        *value = Value::Null;
    }
}
fn scrub_inner(v: &mut Value, lat: &mut Option<f64>, lon: &mut Option<f64>) {
    match v {
        Value::Object(o) => {
            // Detail metrics encode coordinates by descriptor index rather than field name.
            let coords = o
                .get("metricDescriptors")
                .and_then(Value::as_array)
                .map(|ds| {
                    ds.iter()
                        .map(|d| d["key"].as_str().unwrap_or("").to_ascii_lowercase())
                        .collect::<Vec<_>>()
                });
            if let (Some(keys), Some(samples)) = (
                coords,
                o.get_mut("activityDetailMetrics")
                    .and_then(Value::as_array_mut),
            ) {
                for sample in samples {
                    if let Some(values) = sample["metrics"].as_array_mut() {
                        for (i, key) in keys.iter().enumerate() {
                            if let Some(value) = values.get_mut(i) {
                                if key.contains("latitude") {
                                    shift(value, lat)
                                } else if key.contains("longitude") {
                                    shift(value, lon)
                                }
                            }
                        }
                    }
                }
            }
            for (key, val) in o.iter_mut() {
                let k = key.to_ascii_lowercase();
                if k.contains("latitude") || k == "lat" {
                    shift(val, lat)
                } else if k.contains("longitude") || k == "lon" {
                    shift(val, lon)
                } else if k.contains("email") {
                    *val = Value::from("user@example.invalid")
                } else if k.contains("name") {
                    *val = Value::from("Sample User")
                } else if k.contains("token")
                    || k.contains("serial")
                    || k.contains("address")
                    || k.contains("phone")
                {
                    *val = Value::from("REDACTED")
                } else if k.ends_with("id")
                    || k.ends_with("ids")
                    || k.contains("deviceid")
                    || k.contains("identifier")
                    || k.contains("profilepk")
                    || k.contains("uuid")
                    || k.contains("guid")
                    || k == "imei"
                {
                    *val = if val.is_number() {
                        Value::from(123456)
                    } else {
                        Value::from("REDACTED")
                    }
                } else if k.contains("polyline")
                    || k.contains("coordinates")
                    || k.contains("routepoints")
                {
                    if let Some(points) = val.as_array_mut() {
                        for point in points {
                            if let Some(pair) = point.as_array_mut() {
                                if pair.len() >= 2 {
                                    shift(&mut pair[0], lat);
                                    shift(&mut pair[1], lon)
                                } else {
                                    *point = Value::Null;
                                }
                            } else {
                                scrub_inner(point, lat, lon)
                            }
                        }
                    } else if val.is_object() {
                        scrub_inner(val, lat, lon)
                    } else {
                        *val = Value::from("REDACTED")
                    }
                } else {
                    scrub_inner(val, lat, lon)
                }
            }
        }
        Value::Array(a) => {
            for item in a {
                scrub_inner(item, lat, lon)
            }
        }
        _ => {}
    }
}
fn enum_arg<T: serde::de::DeserializeOwned>(s: String) -> Result<T> {
    Ok(serde_json::from_value(Value::String(s))?)
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let now = Local::now();
    let today = now.date_naive().to_string();
    let db = home().join("raring.sqlite");
    if let Some(parent) = db.parent() {
        fs::create_dir_all(parent)?;
    }
    let core = Raring::open(&db)?;
    match args.first().map(String::as_str){
        Some("login")=>{
            let tokens=if args.get(1).is_some_and(|s|s=="--import") {Tokens::import_python(&fs::read_to_string(args.get(2).context("login --import FILE")?)?)?}
            else {let email=prompt("Email: ")?;let password=rpassword::prompt_password("Password: ")?;
                garmin::mobile_login(&email,&password,||prompt("MFA code: ").unwrap_or_default()).await?};
            save_tokens(&tokens)?;println!("Tokens saved with owner-only permissions.");
        },
        Some("sync")=>{let tokens=read_tokens()?;let budget=args.windows(2).find(|v|v[0]=="--budget").map(|v|v[1].parse::<usize>()).transpose()?.unwrap_or(60);
            let result=core.sync(tokens,now,budget).await?;save_tokens(&result.tokens)?;if let Some(error)=&result.snapshot_error{eprintln!("{error}");}show(&result.report)?;},
        Some("status")=>show(&core.status())?,
        Some("heart-rate-day")=>{
            let date=args.get(1).context("heart-rate-day YYYY-MM-DD")?;
            let (day,tokens,report)=core.fetch_heart_rate_day(read_tokens()?,date).await?;
            save_tokens(&tokens)?;if report.auth_failed{bail!("sign in again")};
            if report.rate_limited{bail!("rate limited")};
            if let Some(error)=report.error{bail!("{error}")};
            show(&serde_json::json!({"requests":report.requests,"day":day}))?;
        },
        Some("day-detail")=>{
            let date=args.get(1).context("day-detail YYYY-MM-DD")?;
            let parts=[raring_core::view::DayDetailPart::Sleep,raring_core::view::DayDetailPart::Naps,raring_core::view::DayDetailPart::FitnessAge,raring_core::view::DayDetailPart::Stress,raring_core::view::DayDetailPart::Vitals];
            let (tokens,report)=core.fetch_day_detail(read_tokens()?,date,&parts).await?;
            save_tokens(&tokens)?;if report.auth_failed{bail!("sign in again")};
            if report.rate_limited{bail!("rate limited")};
            if let Some(error)=report.error{bail!("{error}")};
            show(&report)?;
        },
        Some("workout-detail")=>{
            let id=args.get(1).context("workout-detail ACTIVITY_ID")?;
            let (tokens,report)=core.fetch_workout_detail(read_tokens()?,id).await?;
            save_tokens(&tokens)?;if report.auth_failed{bail!("sign in again")};
            if report.rate_limited{bail!("rate limited")};
            if let Some(error)=report.error{bail!("{error}")};
            show(&report)?;
        },
        // A workout's street map, downloading its areas from OpenFreeMap once. With --route FILE (a JSON array of
        // [lat, lon]) it draws a synthetic route in a scratch store instead, as for the preview's sample run.
        Some("route-map")=>{let started=std::time::Instant::now();
            let map=if args.get(1).is_some_and(|a|a=="--route"){
                let route:Vec<[f64;2]>=serde_json::from_str(&fs::read_to_string(args.get(2).context("route-map --route FILE")?)?)?;
                let store=raring_core::store::Store::open(":memory:")?;
                store.put("activity","route","synthetic",&serde_json::json!({"activityId":"route","summaryDTO":{"startTimeLocal":"2025-01-01T00:00:00"}}))?;
                raring_core::maps::fetch(&store,"route",&route,&raring_core::maps::Source::openfreemap(),&raring_core::maps::user_agent(env!("CARGO_PKG_VERSION"))).await?
            }else{core.fetch_route_map(args.get(1).context("route-map ACTIVITY_ID | route-map --route FILE")?,env!("CARGO_PKG_VERSION")).await?};
            eprintln!("route map in {} ms, {} bytes",started.elapsed().as_millis(),serde_json::to_string(&map)?.len());show(&map)?},
        Some("export")=>{let path=Path::new(args.get(1).context("export FILE.zip")?);if path.exists(){bail!("export file already exists")};core.export(path, env!("CARGO_PKG_VERSION"))?;println!("Exported {}",path.display());},
        // Validate an export's settings and, with --apply, import them for this phone's account.
        Some("import")=>{let apply=args.iter().any(|a|a=="--apply");
            let path=Path::new(args.iter().skip(1).find(|a|!a.starts_with("--")).context("import [--apply] FILE.zip")?);
            show(&core.import_preview(path,None)?)?;
            if apply{show(&core.import_settings(path,None)?)?;println!("Imported settings from {}",path.display());}}, 
        // Synthetic QA data for a scratch RARING_HOME; never mixed into a signed-in account's history.
        Some("demo")=>{if core.status().signed_in{bail!("demo needs an empty RARING_HOME, not a signed-in one")}
            let day=match args.windows(2).find(|v|v[0]=="--date"){Some(v)=>chrono::NaiveDate::parse_from_str(&v[1],"%Y-%m-%d").context("--date YYYY-MM-DD")?,None=>now.date_naive()};
            raring_core::demo::seed(&raring_core::store::Store::open(&db)?,day)?;println!("Demo data through {day}.");},
        Some("show")=>{
            // --at YYYY-MM-DDTHH:MM sets the clock, for checking Today's time-of-day phases.
            let now=match args.windows(2).find(|v|v[0]=="--at"){Some(v)=>Local.from_local_datetime(&chrono::NaiveDateTime::parse_from_str(&v[1],"%Y-%m-%dT%H:%M").context("--at YYYY-MM-DDTHH:MM")?).earliest().context("--at: no such local time")?,None=>now};
            let current=core.current_date(now);let date=args.windows(2).find(|v|v[0]=="--date").map(|v|v[1].as_str()).unwrap_or(&current);
            match args.get(1).map(String::as_str){Some("today")=>show(&if args.iter().any(|a|a=="--date") {core.today_at(date,now)} else {core.today_live_at(date,now)})?,Some("sleep")=>show(&core.sleep(date))?,Some("training")=>show(&if args.iter().any(|a|a=="--date") {core.training(date)} else {core.training_live_at(date,now)})?,Some("activities")=>show(&core.activity_month(Some(date),now)?)?,Some("health")=>show(&core.health(date))?,Some("checkin")=>show(&core.checkin(date))?,Some("plan")=>show(&core.plan(date))?,Some("board")=>show(&core.plan_board(date,now)?)?,Some("strength")=>show(&core.strength(date))?,Some("workout")=>show(&core.workout(args.get(2).context("show workout ID")?))?,_=>bail!("show today|sleep|training|activities|health|checkin|plan|board|strength|workout ID")}},
        Some("set-plan")=>{let update=serde_json::from_str(args.get(1).context("set-plan JSON")?)?;core.set_plan(&core.current_date(now),&update)?;show(&core.plan(&core.current_date(now)))?;},
        Some("notifications")=>show(&core.notifications(now))?,
        Some("trend")=>{let current=core.current_date(now);let arg=|k:&str|args.windows(2).find(|v|v[0]==k).map(|v|v[1].clone());
            let query=raring_core::view::TrendQuery{metric:enum_arg(args.get(1).context("trend METRIC")?.clone())?,range:enum_arg(arg("--range").unwrap_or("days30".into()))?,end:arg("--date"),overlay:arg("--overlay").map(enum_arg).transpose()?,previous:args.iter().any(|a|a=="--previous"),start:arg("--start"),comparison_start:arg("--comparison-start")};
            let started=std::time::Instant::now();let view=core.trend(&query,&current).map_err(anyhow::Error::msg)?;eprintln!("trend read in {} ms",started.elapsed().as_millis());show(&view)?},
        Some("stress-month")=>{let current=core.current_date(now);let month=args.get(1).cloned().unwrap_or_else(||current.clone());
            show(&core.stress_month(&month,&current).map_err(anyhow::Error::msg)?)?},
        // One raw read-only request, unscrubbed: for checking an endpoint's shape locally.
        Some("fetch")=>{let kind=args.get(1).context("fetch KIND [KEY]")?;let key=args.get(2).map(String::as_str).unwrap_or("");
            let client=garmin::Client::new(read_tokens()?)?;
            let display=if kind=="daily_hr" {raring_core::store::Store::open(&db)?.get("profile","me").and_then(|v|v["displayName"].as_str().map(str::to_owned))}else{None};
            let result=client.endpoint(kind,key,display.as_deref()).await;save_tokens(&client.tokens().await)?;show(&result?)?;},
        Some("capture")=>{let out=Path::new(args.windows(2).find(|a|a[0]=="--out").map(|a|a[1].as_str()).context("capture --out DIR")?).to_path_buf();
            fs::create_dir_all(&out)?;let client=garmin::Client::new(read_tokens()?)?;
            // Capture explicitly fetches; it is never called during tests or normal sync.
            let mut profile=client.endpoint("profile","me",None).await?;let display=profile["displayName"].as_str().map(str::to_owned);scrub(&mut profile);fs::write(out.join("profile.json"),serde_json::to_vec_pretty(&profile)?)?;
            for kind in ["summary","sleep","hrv","readiness","training","training_daily","load_focus","battery","spo2","respiration","stress","max_metrics","fitness_age","races","endurance","hill","tolerance","records","activities"] {
                let key=if kind=="activities"{"0"}else if kind=="records"{"all"}else{&today};
                match client.endpoint(kind,key,display.as_deref()).await {Ok(mut v)=>{
                    if kind=="activities" {
                        if let Some(id)=v.as_array().and_then(|a|a.first()).and_then(|a|a["activityId"].as_u64().map(|v|v.to_string()).or_else(||a["activityId"].as_str().map(str::to_owned))) {
                            for detail in ["activity","splits","zones","details"] {
                                match client.endpoint(detail,&id,None).await {
                                    Ok(mut value)=>{scrub(&mut value);fs::write(out.join(format!("{detail}.json")),serde_json::to_vec_pretty(&value)?)?;},
                                    Err(e @ (garmin::Error::RateLimited{..} | garmin::Error::NeedsSignIn)) => {save_tokens(&client.tokens().await)?;return Err(e.into())},
                                    Err(e)=>eprintln!("{detail}: {e}")
                                }
                            }
                        }
                    }
                    scrub(&mut v);fs::write(out.join(format!("{kind}.json")),serde_json::to_vec_pretty(&v)?)?;},Err(e @ (garmin::Error::RateLimited{..} | garmin::Error::NeedsSignIn)) => {save_tokens(&client.tokens().await)?;return Err(e.into())},Err(e)=>eprintln!("{kind}: {e}")}
            }
            match client.endpoint("devices", "", None).await {
                Ok(mut list) => {
                    let ids = device_ids(&list);
                    scrub(&mut list);
                    fs::write(out.join("devices.json"), serde_json::to_vec_pretty(&list)?)?;
                    let mut details = Vec::new();
                    for (index, id) in ids.iter().enumerate() {
                        match client.endpoint("device_detail", id, None).await {
                            Ok(mut detail) => {
                                scrub(&mut detail);
                                details.push(serde_json::json!({"listIndex": index, "detail": detail}));
                            }
                            Err(e @ (garmin::Error::RateLimited{..} | garmin::Error::NeedsSignIn)) => {save_tokens(&client.tokens().await)?;return Err(e.into())},
                            Err(e) => eprintln!("device detail #{}: {e}", index + 1),
                        }
                    }
                    fs::write(out.join("device_details.json"), serde_json::to_vec_pretty(&details)?)?;
                }
                Err(e @ (garmin::Error::RateLimited{..} | garmin::Error::NeedsSignIn)) => {save_tokens(&client.tokens().await)?;return Err(e.into())},
                Err(e) => eprintln!("devices: {e}"),
            }
            // A capture can refresh the tokens, and Garmin may rotate the refresh token.
            save_tokens(&client.tokens().await)?;
            println!("Scrubbed capture at {}. Review it manually before sharing.",out.display());
        },
        _=>bail!("usage: raring login [--import FILE] | sync [--budget N] | status | demo [--date YYYY-MM-DD] | show SCREEN [--date YYYY-MM-DD] [--at YYYY-MM-DDTHH:MM] | set-plan JSON | notifications | trend METRIC [--range days7|days30|days90|year|all|custom] [--start DATE] [--date DATE] [--overlay METRIC] [--previous] [--comparison-start DATE] | stress-month [YYYY-MM] | day-detail YYYY-MM-DD | workout-detail ACTIVITY_ID | route-map ACTIVITY_ID|--route FILE | fetch KIND [KEY] | capture --out DIR")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn capture_redacts_identity_and_metric_coordinates() {
        let mut value = json!({"displayName":"Private Person","emailAddress":"real@unsafe.invalid","userProfileId":9876,"token":"secret","metricDescriptors":[{"key":"directLatitude"},{"key":"directLongitude"}],"activityDetailMetrics":[{"metrics":[42.5,-73.2]},{"metrics":[42.5001,-73.1999]}]});
        scrub(&mut value);
        assert_eq!(value["displayName"], "Sample User");
        assert_eq!(value["metricDescriptors"][0]["key"], "directLatitude");
        assert_eq!(
            value["activityDetailMetrics"][0]["metrics"],
            json!([0.0, 0.0])
        );
        assert_eq!(
            value["activityDetailMetrics"][1]["metrics"],
            json!([0.0001, 0.0001])
        );
        let text = value.to_string();
        assert!(!text.contains("Private Person"));
        assert!(!text.contains("real@unsafe.invalid"));
        assert!(!text.contains("42.5"));
        let mut geometry =
            json!({"polyline":[[37.77,-122.42],[37.771,-122.419]],"encodedPolyline":"realroute"});
        scrub(&mut geometry);
        assert_eq!(geometry["polyline"][0], json!([0.0, 0.0]));
        assert_eq!(geometry["polyline"][1], json!([0.001, 0.001]));
        assert_eq!(geometry["encodedPolyline"], "REDACTED");
    }

    #[test]
    fn capture_redacts_nested_device_identifiers_and_serials() {
        let mut list = json!([{
            "deviceId": "AB12-DEV", "serialNumber": "SN-9XZ42",
            "unitId": 938472, "lastUploadTimestamp": "2026-10-17T08:30:00Z",
            "capabilities": [{"deviceIdentifier": "hidden-identifier", "uuid": "uuid-secret",
                "nested": {"hardwareSerial": "hardware-secret", "guid": "guid-secret"}}]
        }]);
        assert_eq!(device_ids(&list), ["AB12-DEV"]);
        scrub(&mut list);
        let text = list.to_string();
        for secret in [
            "AB12-DEV",
            "SN-9XZ42",
            "938472",
            "hidden-identifier",
            "uuid-secret",
            "hardware-secret",
            "guid-secret",
        ] {
            assert!(!text.contains(secret), "leaked device identifier: {secret}");
        }
        assert_eq!(list[0]["deviceId"], "REDACTED");
        assert_eq!(list[0]["unitId"], 123456);
        assert_eq!(list[0]["lastUploadTimestamp"], "2026-10-17T08:30:00Z");
        let mut detail = json!({"userDeviceId": "another-device", "nested": [
            {"serialNumbers": ["serial-1", "serial-2"], "deviceID": "alpha-99",
             "deviceIds": ["plural-1", "plural-2"]},
            {"device": {"deviceId": 752989, "parentIdentifier": "parent-7"}}
        ]});
        scrub(&mut detail);
        let text = detail.to_string();
        for secret in [
            "another-device",
            "serial-1",
            "serial-2",
            "alpha-99",
            "plural-1",
            "plural-2",
            "752989",
            "parent-7",
        ] {
            assert!(!text.contains(secret), "leaked detail identifier: {secret}");
        }
        assert_eq!(device_ids(&json!({"devices": [{"deviceId": 42}]})), ["42"]);
    }
}
