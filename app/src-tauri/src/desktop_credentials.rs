//! One desktop credential gate for every read/refresh/write and account change.
//! Android uses the existing vault's SyncGate instead.
use raring_core::Tokens;
use std::{
    future::Future,
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct DesktopCredentials {
    gate: Mutex<()>,
    generation: AtomicU64,
}

impl DesktopCredentials {
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }
    pub fn invalidate_sign_in(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Keep the lock from the disk read through the network request and token
    /// write. The operation returns rotated tokens even when its endpoint
    /// reported a data failure; the caller examines that failure *after* save.
    pub async fn use_tokens<T, F, Fut>(
        &self,
        read: impl FnOnce() -> Result<Option<Tokens>, String>,
        save: impl FnOnce(&Tokens) -> Result<(), String>,
        operation: F,
    ) -> Result<Option<T>, String>
    where
        F: FnOnce(Tokens) -> Fut,
        Fut: Future<Output = Result<(T, Tokens), String>>,
    {
        let _guard = self.gate.lock().await;
        let Some(tokens) = read()? else {
            return Ok(None);
        };
        let (result, rotated) = operation(tokens).await?;
        save(&rotated)?;
        Ok(Some(result))
    }

    pub async fn save_sign_in(
        &self,
        generation: u64,
        save: impl FnOnce() -> Result<(), String>,
    ) -> Result<bool, String> {
        let _guard = self.gate.lock().await;
        if self.generation() != generation {
            return Ok(false);
        }
        save()?;
        Ok(true)
    }

    pub async fn sign_out(&self, clear: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
        self.invalidate_sign_in();
        let _guard = self.gate.lock().await;
        clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use raring_core::{garmin::Client, store::Store, sync};
    use serde_json::json;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        path::Path,
        sync::{Arc, Mutex as StdMutex},
    };

    fn read(path: &Path) -> Result<Option<Tokens>, String> {
        std::fs::read_to_string(path)
            .map_err(|e| e.to_string())
            .and_then(|value| {
                serde_json::from_str(&value)
                    .map(Some)
                    .map_err(|e| e.to_string())
            })
    }
    fn save(path: &Path, tokens: &Tokens) -> Result<(), String> {
        std::fs::write(path, serde_json::to_vec(tokens).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn overlapping_refresh_and_failed_endpoint_preserve_rotated_tokens() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let dir = std::env::temp_dir().join(format!("raring-credential-gate-{}-{}", std::process::id(), Utc::now().timestamp_nanos_opt().unwrap()));
            std::fs::create_dir(&dir).unwrap();
            let path = dir.join("tokens.json");
            let jwt = |exp| format!("e30.{}.sig", base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, json!({"exp":exp,"client_id":"TEST_DI"}).to_string()));
            let old = jwt(1_000_000_000);
            let rotated = jwt(2_000_000_000);
            save(&path, &Tokens { access_token: old, refresh_token:"old-refresh".into(), client_id:"TEST_DI".into(), expires_at: Utc::now()-Duration::hours(1) }).unwrap();
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let seen = Arc::new(StdMutex::new(Vec::new()));
            let recorded = Arc::clone(&seen);
            let server = std::thread::spawn(move || {
                for _ in 0..3 {
                    let (mut stream, _) = listener.accept().unwrap();
                    let mut buf = [0;8192];
                    let n = stream.read(&mut buf).unwrap();
                    let request = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let (status, body) = if request.starts_with("POST ") {
                        std::thread::sleep(std::time::Duration::from_millis(80));
                        (200,json!({"access_token":rotated,"refresh_token":"rotated-refresh"}).to_string())
                    } else if request.contains("date=2025-06-03") { (500,"{}".into()) }
                    else { (200,"{}".into()) };
                    recorded.lock().unwrap().push(request);
                    let response = format!("HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
                    stream.write_all(response.as_bytes()).unwrap();
                }
            });
            let gate=Arc::new(DesktopCredentials::default());
            let store=Arc::new(Store::open(":memory:").unwrap());
            store.put("profile","me","synthetic",&json!({"displayName":"Synthetic"})).unwrap();
            let mut tasks=Vec::new();
            for date in ["2025-06-03","2025-06-04"] {
                let gate=Arc::clone(&gate);let store=Arc::clone(&store);let path=path.clone();let url=url.clone();
                tasks.push(tokio::spawn(async move {
                    gate.use_tokens(||read(&path),|tokens|save(&path,tokens),|tokens|async move {
                        let client=Client::with_urls(tokens,&url,&url,std::time::Duration::ZERO).map_err(|e|e.to_string())?;
                        let report=sync::heart_rate_on_demand(&store,&client,date).await;
                        Ok((report,client.tokens().await))
                    }).await.unwrap().unwrap()
                }));
            }
            let a=tasks.remove(0).await.unwrap();let b=tasks.remove(0).await.unwrap();
            server.join().unwrap();
            assert_eq!(a.data_failures+b.data_failures,1);
            assert_eq!(a.requests+b.requests,2);
            let requests=seen.lock().unwrap();
            assert_eq!(requests.iter().filter(|r|r.starts_with("POST ")).count(),1,"only one refresh");
            assert_eq!(requests.iter().filter(|r|r.contains("authorization: Bearer ")).count(),2);
            let persisted=read(&path).unwrap().unwrap();
            assert_eq!(persisted.refresh_token,"rotated-refresh");
            assert!(requests.iter().filter(|r|r.starts_with("GET ")).all(|r|r.contains(&format!("authorization: Bearer {}",persisted.access_token))));
            std::fs::remove_dir_all(dir).unwrap();
        });
    }

    #[test]
    fn sign_out_and_new_sign_in_cannot_be_reversed_by_an_old_save() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let gate = Arc::new(DesktopCredentials::default());
            let old = gate.invalidate_sign_in();
            let gate_for_request = Arc::clone(&gate);
            let (started, ready) = tokio::sync::oneshot::channel();
            let operation = tokio::spawn(async move {
                gate_for_request
                    .use_tokens(
                        || {
                            Ok(Some(Tokens {
                                access_token: "old".into(),
                                refresh_token: "old".into(),
                                client_id: "TEST_DI".into(),
                                expires_at: Utc::now(),
                            }))
                        },
                        |_| {
                            let _ = started.send(());
                            Ok(())
                        },
                        |tokens| async move {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                            Ok(((), tokens))
                        },
                    )
                    .await
                    .unwrap();
            });
            // The old request owns the gate before sign-out is queued.
            tokio::task::yield_now().await;
            let signed_out = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let flag = Arc::clone(&signed_out);
            gate.sign_out(|| {
                flag.store(true, Ordering::SeqCst);
                Ok(())
            })
            .await
            .unwrap();
            operation.await.unwrap();
            ready.await.unwrap();
            assert!(signed_out.load(Ordering::SeqCst));
            let new = gate.invalidate_sign_in();
            assert!(!gate
                .save_sign_in(old, || panic!("old sign-in must not save"))
                .await
                .unwrap());
            assert!(gate.save_sign_in(new, || Ok(())).await.unwrap());
        });
    }
}
