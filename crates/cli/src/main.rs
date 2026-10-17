use anyhow::{bail, Context, Result};
use chrono::Local;
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
                } else if k.ends_with("id") || k.contains("profilepk") || k.contains("userid") {
                    *val = Value::from("123456")
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
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let now = Local::now();
    let today = now.date_naive().to_string();
    let db = home().join("raring.sqlite");
    if let Some(parent) = db.parent() {
        fs::create_dir_all(parent)?;
    }
    let core = Raring::open(db)?;
    match args.first().map(String::as_str){
        Some("login")=>{
            let tokens=if args.get(1).is_some_and(|s|s=="--import") {Tokens::import_python(&fs::read_to_string(args.get(2).context("login --import FILE")?)?)?}
            else {let email=prompt("Email: ")?;let password=rpassword::prompt_password("Password: ")?;
                garmin::mobile_login(&email,&password,||prompt("MFA code: ").unwrap_or_default()).await?};
            save_tokens(&tokens)?;println!("Tokens saved with owner-only permissions.");
        },
        Some("sync")=>{let tokens=read_tokens()?;let budget=args.windows(2).find(|v|v[0]=="--budget").map(|v|v[1].parse::<usize>()).transpose()?.unwrap_or(60);
            let result=core.sync(tokens,now,budget).await?;save_tokens(&result.tokens)?;show(&result.report)?;},
        Some("status")=>show(&core.status())?,
        Some("show")=>{let current=core.current_date(now);let date=args.windows(2).find(|v|v[0]=="--date").map(|v|v[1].as_str()).unwrap_or(&current);
            match args.get(1).map(String::as_str){Some("today")=>show(&core.today(date))?,Some("sleep")=>show(&core.sleep(date))?,Some("training")=>show(&core.training(date))?,Some("health")=>show(&core.health(date))?,Some("checkin")=>show(&core.checkin(date))?,Some("workout")=>show(&core.workout(args.get(2).context("show workout ID")?))?,_=>bail!("show today|sleep|training|health|checkin|workout ID")}},
        Some("notifications")=>show(&core.notifications(now))?,
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
                                    Err(e)=>eprintln!("{detail}: {e}")
                                }
                            }
                        }
                    }
                    scrub(&mut v);fs::write(out.join(format!("{kind}.json")),serde_json::to_vec_pretty(&v)?)?;},Err(e)=>eprintln!("{kind}: {e}")}
            }
            println!("Scrubbed capture at {}. Review it manually before sharing.",out.display());
        },
        _=>bail!("usage: raring login [--import FILE] | sync [--budget N] | status | show SCREEN [--date YYYY-MM-DD] | notifications | capture --out DIR")
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
}
