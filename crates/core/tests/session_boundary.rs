use chrono::{Duration, Utc};
use raring_core::{account, garmin::Client, store::Store, Raring, Tokens};
use serde_json::json;
use std::{fs, io::{Read, Write}, net::TcpListener, sync::{mpsc, Arc}, time::Duration as Wait};

fn tokens() -> Tokens {
    Tokens { access_token:"synthetic".into(), refresh_token:"synthetic".into(), client_id:"synthetic".into(), expires_at:Utc::now()+Duration::hours(1) }
}
fn fixture(name: &str) -> (std::path::PathBuf, Arc<Store>, Arc<Raring>) {
    let dir = std::env::temp_dir().join(format!("raring-session-{name}-{}",std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let db=dir.join("raring.db");
    let store=Arc::new(Store::open(&db).unwrap());
    store.put("profile","me","synthetic",&json!({"profileId":101,"displayName":"fake-A"})).unwrap();
    store.set_setting("account_session", &json!({"identity":"101","active":true}).to_string()).unwrap();
    store.set_setting("plan","A targets").unwrap();
    (dir,store,Arc::new(Raring::open(db).unwrap()))
}
fn server() -> (String, mpsc::Receiver<()>, mpsc::Sender<()>, std::thread::JoinHandle<()>) {
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();
    let url=format!("http://{}",listener.local_addr().unwrap());
    let (requested,request)=mpsc::channel();
    let (release,reply)=mpsc::channel();
    let thread=std::thread::spawn(move||{
        let (mut socket,_)=listener.accept().unwrap();
        let mut buf=[0;2048]; let n=socket.read(&mut buf).unwrap();
        assert!(String::from_utf8_lossy(&buf[..n]).starts_with("GET /userprofile-service/socialProfile "));
        requested.send(()).unwrap(); reply.recv().unwrap();
        let body=r#"{"profileId":202,"displayName":"fake-B"}"#;
        socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).unwrap();
    });
    (url,request,release,thread)
}

#[test]
fn account_switch_cannot_cross_local_plan_mutation_or_settings_save() {
    let (dir,store,core)=fixture("mutation");
    let (entered,started)=mpsc::channel(); let (release,proceed)=mpsc::channel();
    let view_core=Arc::clone(&core); let view_store=Arc::clone(&store);
    let view=std::thread::spawn(move||view_core.with_verified_session("101", ||{
        let plan=view_store.setting("plan").unwrap();
        entered.send(()).unwrap(); proceed.recv().unwrap();
        // Represents the read-modify-write done by local plans/view plan capture.
        view_store.set_setting("plan",&format!("{plan} edited")).unwrap();
    }));
    started.recv().unwrap();
    let old_settings=core.settings(); let old_identity=core.bound_account_identity();
    let (url,request,reply,server)=server();
    let switched_store=Arc::clone(&store); let (done,finished)=mpsc::channel();
    let switch=std::thread::spawn(move||{
        let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let client=Client::with_urls(tokens(),&url,&url,Wait::ZERO).unwrap();
            account::verify(&switched_store,&client,Utc::now()).await.unwrap();
            account::activate(&switched_store).unwrap();
        });
        done.send(()).unwrap();
    });
    assert!(finished.recv_timeout(Wait::from_millis(100)).is_err(),"transition must wait for the local mutation");
    release.send(()).unwrap(); view.join().unwrap().unwrap();
    request.recv().unwrap(); reply.send(()).unwrap();
    switch.join().unwrap(); server.join().unwrap();
    assert_eq!(core.session_identity().as_deref(),Some("202"));
    assert!(store.setting("plan").is_none(),"A's mutation must finish before B's erase");
    assert!(core.with_verified_session("101", || store.set_setting("plan", "queued A edit")).is_none(),
        "a queued mutation must validate its expected identity inside the barrier");
    assert!(store.setting("plan").is_none());
    assert!(core.with_verified_session("101", || core.status()).is_none(),
        "old-account status metadata must not be authorized by the new active session");
    assert!(core.set_settings_for_account(old_settings,old_identity).is_err(),"a queued A settings save must not restore A's flags into B");
    drop(core);drop(store);fs::remove_dir_all(dir).unwrap();
}

#[test]
fn profile_network_wait_does_not_hold_the_local_barrier() {
    let (dir,store,core)=fixture("network");
    let (url,request,reply,server)=server();
    let verifying=Arc::clone(&store);
    let verify=std::thread::spawn(move||{
        let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let client=Client::with_urls(tokens(),&url,&url,Wait::ZERO).unwrap();
            account::verify(&verifying,&client,Utc::now()).await.unwrap();
        });
    });
    request.recv().unwrap(); // The server deliberately holds its response.
    let settings_core=Arc::clone(&core); let (done,saved)=mpsc::channel();
    let settings=std::thread::spawn(move||{
        settings_core.set_settings_for_account(Default::default(),Some("101".into())).unwrap();
        done.send(()).unwrap();
    });
    assert!(saved.recv_timeout(Wait::from_secs(2)).is_ok(),"network wait must not block a local opt-out");
    reply.send(()).unwrap();settings.join().unwrap();verify.join().unwrap();server.join().unwrap();
    drop(core);drop(store);fs::remove_dir_all(dir).unwrap();
}
