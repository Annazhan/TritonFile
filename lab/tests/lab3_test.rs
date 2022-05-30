use bincode::options;
use clap::{crate_version, Arg, Command};
use front::client_fs::lab::{new_bin_client, serve_back};
use front::client_fs::{client::new_client, front::Front};
use fuser::{
    Filesystem, KernelConfig, MountOption, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyEmpty, ReplyEntry, ReplyOpen, ReplyStatfs, ReplyWrite, ReplyXattr, Request, TimeOrNow,
    FUSE_ROOT_ID,
};
use log::{error, LevelFilter};
#[allow(unused_imports)]
use rand::{prelude::SliceRandom, Rng};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, ErrorKind, Read, Seek, SeekFrom, Write};
use std::{
    sync::{mpsc, Arc},
    time::{Duration, SystemTime},
    vec,
};
use tokio::{runtime::Handle, time};
use tribbler::config::KeeperConfig;
use tribbler::{
    self,
    config::BackConfig,
    error::{TritonFileError, TritonFileResult},
    storage::{KeyValue, Pattern, RemoteFileSystem},
};

const KEY_KEEPER: &str = "KEEPER";
const KEY_TIMESTAMP: &str = "TIMESTAMP";
fn kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: value.to_string(),
    }
}

fn compose_timestamp_key(idx: usize) -> String {
    format!("{}_{}", KEY_TIMESTAMP, idx)
}

async fn get_backend_info(backs: Vec<String>, is_keeper: bool) -> TritonFileResult<()> {
    println!("get backend info");
    let p = Pattern {
        prefix: "".to_string(),
        suffix: "".to_string(),
    };

    for i in 0..backs.len() {
        println!("\n current backend:{}", backs[i].clone());
        let client = new_client(backs[i].clone().as_str()).await;
        match client {
            Ok(cli) => {
                let mut all_string_keys = Vec::new();
                let key_res = match is_keeper {
                    true => cli.keys(&p).await,
                    false => cli.list_keys(&p).await,
                };
                match key_res {
                    Ok(keys) => all_string_keys = keys.0,
                    Err(_) => {
                        println!("this backend is not alive");
                        continue;
                    }
                }

                all_string_keys.sort();
                println!("all string keys: {:?}", all_string_keys);
                for k in all_string_keys {
                    match is_keeper {
                        true => match cli.get(&k).await {
                            Ok(value) => {
                                println!("{}:{:?}", k, value);
                            }
                            Err(_) => {
                                println!("this backend is not alive");
                            }
                        },
                        false => match cli.list_get(&k).await {
                            Ok(value) => {
                                println!("{}:{:?}", k, value.0);
                            }
                            Err(_) => {
                                println!("this backend is not alive");
                            }
                        },
                    }
                }
            }
            Err(_) => {
                println!("this backend is not alive");
                continue;
            }
        }
    }

    Ok(())
}

/// total number of back end and keeper
fn generate_addr(
    back_len: usize,
    keeper_len: usize,
    used_back: usize,
    used_kp: usize,
) -> (
    (Vec<String>, Vec<String>, Vec<String>),
    (Vec<String>, Vec<String>, Vec<String>),
) {
    let mut backs = vec![];
    for i in 32314..(32314 + back_len) {
        backs.push(format!("127.0.0.1:{}", i));
    }

    let mut used_back_addr = vec![];
    let mut unused_back_addr = vec![];
    let mut rng = rand::thread_rng();
    let mut used_index = rand::seq::index::sample(&mut rng, back_len, used_back).into_vec();

    for i in 0..backs.len() {
        if used_index.contains(&i) {
            used_back_addr.push(backs[i].clone());
        } else {
            unused_back_addr.push(backs[i].clone());
        }
    }

    let mut keepers = vec![];
    for j in (32314 + back_len)..(32314 + back_len + keeper_len) {
        keepers.push(format!("127.0.0.1:{}", j));
    }

    used_index = rand::seq::index::sample(&mut rng, keeper_len, used_kp).into_vec();
    let mut used_keeper_addr = vec![];
    let mut unused_keeper_addr = vec![];
    for i in 0..keeper_len {
        if used_index.contains(&i) {
            used_keeper_addr.push(keepers[i].clone());
        } else {
            unused_keeper_addr.push(keepers[i].clone());
        }
    }

    return (
        (backs, used_back_addr, unused_back_addr),
        (keepers, used_keeper_addr, unused_keeper_addr),
    );
}

fn spawn_back(cfg: BackConfig) -> tokio::task::JoinHandle<TritonFileResult<()>> {
    tokio::spawn(serve_back(cfg))
}

// fn spawn_keep(kfg: KeeperConfig) -> tokio::task::JoinHandle<TritonFileResult<()>> {
//     tokio::spawn(lab3::serve_keeper(kfg))
// }

// async fn spawn_front_follow(who: &str, whom: &str, back_addr: Vec<String>) -> TritonFileResult<()> {
//     match generate_client(back_addr).await {
//         Ok(front) => front.follow(who, whom).await,
//         Err(e) => Err(Box::new(TritonFileError::Unknown(e.to_string()))),
//     }
// }

// async fn spawn_front_unfollow(
//     who: &str,
//     whom: &str,
//     back_addr: Vec<String>,
// ) -> TritonFileResult<()> {
//     match generate_client(back_addr).await {
//         Ok(front) => front.unfollow(who, whom).await,
//         Err(e) => Err(Box::new(TritonFileError::Unknown(e.to_string()))),
//     }
// }

// async fn spawn_front_post(who: Arc<String>, back_addr: Vec<String>) -> TritonFileResult<()> {
//     match generate_client(back_addr).await {
//         Ok(front) => {
//             front
//                 .post(&format!("{}", who), &generate_random_string(20), 10)
//                 .await
//         }
//         Err(e) => Err(Box::new(TritonFileError::Unknown(e.to_string()))),
//     }
// }

fn generate_random_string(len: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();

    let password: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    return password;
}

// All the available backends and keepers and the number of lives
// the number of keepers is at least 3
async fn set_up_backs(
    backs: Vec<String>,
) -> TritonFileResult<
    Vec<(
        String,
        tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
        tokio::sync::mpsc::Sender<()>,
    )>,
> {
    let mut back_ends = vec![];
    for i in 0..backs.len() {
        let (sx, rx) = mpsc::channel();
        let (shut_sx, shut_rx) = tokio::sync::mpsc::channel(1);
        let cfg = BackConfig {
            addr: backs[i].clone(),
            storage: Box::new(RemoteFileSystem::new(i.try_into().unwrap())),
            ready: Some(sx.clone()),
            shutdown: Some(shut_rx),
        };
        let handle = spawn_back(cfg);
        let ready = rx.recv_timeout(Duration::from_secs(10))?;
        if !ready {
            return Err(Box::new(TritonFileError::Unknown(
                "back fail to start".to_string(),
            )));
        }

        // println!("The backend {} is start", backs[i]);
        back_ends.push((backs[i].clone(), handle, shut_sx.clone()));
    }

    println!("the number of backend {}", back_ends.len());
    return Ok(back_ends);
}

// async fn set_up_kp(
//     backs: Vec<String>,
//     keepers: Vec<String>,
//     used_keeper: Vec<String>,
// ) -> TritonFileResult<
//     Vec<(
//         String,
//         tokio::task::JoinHandle<Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>>,
//         tokio::sync::mpsc::Sender<()>,
//     )>,
// > {
//     let mut keeper_info = vec![];
//     for i in 0..used_keeper.len() {
//         let (sx, rx) = mpsc::channel();
//         let (shut_sx, shut_rx) = tokio::sync::mpsc::channel(1);
//         let kp_cfg = KeeperConfig {
//             backs: backs.clone(),
//             addrs: keepers.clone(),
//             this: i,
//             id: 0,
//             ready: Some(sx.clone()),
//             shutdown: Some(shut_rx),
//         };

//         let handle = spawn_keep(kp_cfg);
//         match rx.recv_timeout(Duration::from_secs(5)) {
//             Ok(_) => {}
//             Err(_) => {
//                 return Err(Box::new(TritonFileError::Unknown(
//                     "keeper fail to start".to_string(),
//                 )));
//             }
//         }
//         keeper_info.push((keepers[i].clone(), handle, shut_sx.clone()));
//     }

//     return Ok(keeper_info);
// }

// async fn generate_client(
//     back_addr: Vec<String>,
// ) -> TritonFileResult<Box<dyn Filesystem + Send + Sync>> {
//     let bin_client = new_bin_client(back_addr).await?;
//     let front = new_front(bin_client).await?;
//     return Ok(front);
// }

async fn shut_down_all(
    backs: Vec<(
        String,
        tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
        tokio::sync::mpsc::Sender<()>,
    )>,
    kps: Vec<(
        String,
        tokio::task::JoinHandle<Result<(), Box<(dyn std::error::Error + Send + Sync + 'static)>>>,
        tokio::sync::mpsc::Sender<()>,
    )>,
) {
    for (_, handle, _) in backs {
        handle.abort();
    }

    for (_, handle, _) in kps {
        handle.abort();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_server_setup() -> TritonFileResult<()> {
    let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
        generate_addr(6, 4, 4, 3);
    let mut backs = set_up_backs(used_back_addr.clone()).await?;

    let bin_client = new_bin_client(back_addr).await?;

    let mut options = vec![MountOption::FSName(format!("fuser"))];
    #[cfg(feature = "abi-7-26")]
    {
        options.push(MountOption::AutoUnmount);
    }
    #[cfg(not(feature = "abi-7-26"))]
    {
        options.push(MountOption::AutoUnmount);
        options.push(MountOption::AllowOther);
        options.push(MountOption::Dev);
        options.push(MountOption::AllowRoot);
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
    .build()
    .unwrap();
    let front =  Front::new(
        bin_client,
        runtime
    );
    let result = fuser::mount2(
        front, 
        "/Users/stella/Desktop/tmp/",
        &options,
    );
    if let Err(e) = result {
        // Return a special error code for permission denied, which usually indicates that
        // "user_allow_other" is missing from /etc/fuse.conf
        if e.kind() == ErrorKind::PermissionDenied {
            error!("{}", e.to_string());
            std::process::exit(2);
        }
    }
    Ok(())
}

fn fuse_allow_other_enabled() -> io::Result<bool> {
    let file = File::open("/etc/fuse.conf")?;
    for line in BufReader::new(file).lines() {
        if line?.trim_start().starts_with("user_allow_other") {
            return Ok(true);
        }
    }
    Ok(false)
}

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_server_shutdown_channel() -> TritonFileResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let mut kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr).await?;
//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     let (remove_addr, handle, shut_sx) = backs.remove(2);
//     let client = new_client(&remove_addr).await?;
//     // assert!(client.set(&kv("hello", "hi")).await?);
//     shut_sx.send(()).await;
//     let r = handle.await.unwrap();
//     assert!(r.is_ok());

//     match client.get("hello").await {
//         Ok(_) => panic!("the server doesn't shut down"),
//         Err(_) => (),
//     }
//     let _ = shut_down_all(backs, kps);
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_server_shutdown_tolerent() -> TritonFileResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let mut kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;
//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     let (remove_addr, handle, shut_sx) = backs.remove(2);
//     shut_sx.send(()).await;
//     let r = handle.await.unwrap();
//     assert!(r.is_ok());

//     let tribs = front.tribs("h8liu").await?;
//     assert_eq!("Hello, world.", tribs[0].message);
//     let _ = shut_down_all(backs, kps);
//     return Ok(());
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_server_multi_shutdown_tolerent() -> TritonFileResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(8, 4, 5, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;
//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     let (remove_addr, handle, shut_sx) = backs.remove(2);
//     let _ = shut_sx.send(()).await;
//     let r = handle.await.unwrap();
//     assert!(r.is_ok());

//     let wait = time::sleep_until(time::Instant::now() + time::Duration::from_secs(30));
//     wait.await;

//     let (remove_addr, handle, shut_sx) = backs.remove(2);
//     let _ = shut_sx.send(()).await;
//     let r = handle.await.unwrap();
//     assert!(r.is_ok());

//     let tribs = front.tribs("h8liu").await?;
//     assert_eq!("Hello, world.", tribs[0].message);

//     let _ = shut_down_all(backs, kps);
//     return Ok(());
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_server_join() -> TritonFileResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let backs = set_up_backs(used_back_addr.clone()).await?;
//     let kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;

//     get_backend_info(back_addr.clone(), false).await?;

//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;
//     front.follow("h8liu", "fenglu").await?;

//     // get_backend_info(back_addr.clone()).await?;

//     // println!("unused_back_end {:?}", unused_back_addr);
//     let new_back_addr = unused_back_addr.choose(&mut rand::thread_rng()).unwrap();

//     // println!("The address of new back end is {}", new_back_addr);
//     let _ = set_up_backs(vec![new_back_addr.to_string()]).await?;
//     // backs.append(&mut new_backs.clone());
//     get_backend_info(back_addr, false).await?;
//     // println!("check the home of fenglu");
//     let user_home = front.home("fenglu").await?;

//     // println!(
//     //     // "The first trib on the home page is {}",
//     //     user_home[0].message
//     // );
//     assert_eq!("Double tribble.", user_home[0].message);
//     let _ = shut_down_all(backs, kps);
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_server_revive() -> TritonFileResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;
//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;
//     front.follow("h8liu", "fenglu").await?;

//     // get_backend_info(back_addr.clone()).await;
//     let (remove_addr, handle, shut_sx) = backs.remove(2);
//     let _ = shut_sx.send(()).await;
//     let r = handle.await.unwrap();
//     assert!(r.is_ok());
//     // println!("1. check the Trib of h8liu");
//     let tribs = front.tribs("h8liu").await?;
//     assert_eq!("Hello, world.", tribs[0].message);
//     let wait = time::sleep_until(time::Instant::now() + time::Duration::from_secs(30));
//     wait.await;

//     let mut new_backs = set_up_backs(vec![remove_addr]).await?;
//     // get_backend_info(back_addr.clone()).await;
//     // println!("2. check the Trib of h8liu");
//     let tribs = front.tribs("h8liu").await?;
//     assert_eq!("Hello, world.", tribs[0].message);
//     // get_backend_info(back_addr.clone()).await;
//     let _ = shut_down_all(backs, kps);
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_operation_in_server_revive() -> TribResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;
//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;
//     front.follow("h8liu", "fenglu").await?;

//     let (remove_addr, handle, shut_sx) = backs.remove(2);
//     let _ = shut_sx.send(()).await;
//     let r = handle.await.unwrap();
//     assert!(r.is_ok());
//     let tribs = front.tribs("h8liu").await?;
//     assert_eq!("Hello, world.", tribs[0].message);

//     front.post("h8liu", "Thrid tribble", 5).await?;

//     let wait = time::sleep_until(time::Instant::now() + time::Duration::from_secs(30));
//     wait.await;
//     let _ = set_up_backs(vec![remove_addr]).await?;
//     let tribs = front.tribs("h8liu").await?;
//     assert_eq!("Thrid tribble", tribs[2].message);

//     let _ = shut_down_all(backs, kps);
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_multi_thread_follow_server_revive() -> TribResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;
//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     let (remove_addr, handle, shut_sx) = backs.remove(2);
//     let _ = shut_sx.send(()).await;
//     let r = handle.await.unwrap();
//     assert!(r.is_ok());
//     // println!("1. check the Trib of h8liu");
//     let tribs = front.tribs("h8liu").await?;
//     assert_eq!("Hello, world.", tribs[0].message);

//     let mut handles = vec![];
//     for _ in 0..5 {
//         handles.push(tokio::spawn(spawn_front_follow(
//             "h8liu",
//             "fenglu",
//             back_addr.clone(),
//         )));
//     }

//     let wait = time::sleep_until(time::Instant::now() + time::Duration::from_secs(30));
//     wait.await;
//     let _ = set_up_backs(vec![remove_addr]).await?;

//     let mut count = 0;
//     for h in handles {
//         match h.await {
//             Ok(ret) => match ret {
//                 Ok(_) => {
//                     count += 1;
//                     if count > 1 {
//                         panic!("follow success mutilple times");
//                     }
//                 }
//                 Err(_) => {}
//             },
//             Err(_) => {}
//         }
//     }

//     assert_eq!(true, front.is_following("h8liu", "fenglu").await?);

//     let _ = shut_down_all(backs, kps);
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_multi_thread_unfollow_server_revive() -> TribResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;
//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     let (remove_addr, handle, shut_sx) = backs.remove(2);
//     let _ = shut_sx.send(()).await;
//     let r = handle.await.unwrap();
//     assert!(r.is_ok());
//     // println!("1. check the Trib of h8liu");
//     let tribs = front.tribs("h8liu").await?;
//     assert_eq!("Hello, world.", tribs[0].message);

//     let mut handles = vec![];
//     for _ in 0..5 {
//         handles.push(tokio::spawn(spawn_front_unfollow(
//             "h8liu",
//             "fenglu",
//             back_addr.clone(),
//         )));
//     }

//     let wait = time::sleep_until(time::Instant::now() + time::Duration::from_secs(30));
//     wait.await;

//     let _ = set_up_backs(vec![remove_addr]).await?;

//     let mut count = 0;
//     for h in handles {
//         match h.await {
//             Ok(ret) => match ret {
//                 Ok(_) => {
//                     count += 1;
//                     if count > 1 {
//                         panic!("follow success mutilple times");
//                     }
//                 }
//                 Err(_) => {}
//             },
//             Err(_) => {}
//         }
//     }

//     assert_eq!(false, front.is_following("h8liu", "fenglu").await?);
//     let _ = shut_down_all(backs, kps);
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_front_home_perforamance() -> TribResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;
//     let front = generate_client(back_addr.clone()).await?;
//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     for i in 0..10 {
//         let name = generate_random_string(9);
//         front.sign_up(&name).await?;
//         front.follow("h8liu", &name).await?;
//         for j in 0..100 {
//             front
//                 .post(&name, &generate_random_string(20), i * 10 + j)
//                 .await?;
//         }
//     }

//     let pre = SystemTime::now();
//     let handle = tokio::spawn(async move { front.home("h8liu").await });
//     time::sleep(time::Duration::from_secs(7)).await;

//     handle.abort();

//     match handle.await {
//         Ok(_) => {}
//         Err(e) => match e.is_cancelled() {
//             true => {
//                 assert!(false, "function Home takes to long Error");
//             }
//             false => {}
//         },
//     }

//     shut_down_all(backs, kps).await;
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_sign_up_perforamance() -> TribResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let backs = set_up_backs(used_back_addr.clone()).await?;
//     let kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;

//     let mut handles = vec![];

//     for _ in 0..100 {
//         let name = generate_random_string(9);
//         let front = generate_client(back_addr.clone()).await?;
//         handles.push(tokio::spawn(async move { front.sign_up(&name).await }));
//     }
//     time::sleep(time::Duration::from_secs(5)).await;

//     for h in handles {
//         h.abort();

//         match h.await {
//             Ok(_) => {}
//             Err(e) => match e.is_cancelled() {
//                 true => {
//                     assert!(false, "function Home takes to long Error");
//                 }
//                 false => {
//                     println!("The error is {}", e);
//                 }
//             },
//         }
//     }

//     shut_down_all(backs, kps).await;
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_max_server_perforamance() -> TribResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 3, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let mut kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr.clone()).await?;
//     let front = generate_client(back_addr.clone()).await?;

//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     let mut users = vec![];
//     let mut handles = vec![];

//     for _ in 0..5 {
//         let name = generate_random_string(9);
//         users.push(Arc::new(name.clone()));
//         let front = generate_client(back_addr.clone()).await?;
//         handles.push(tokio::spawn(async move { front.sign_up(&name).await }));
//     }

//     for h in handles {
//         match h.await {
//             Ok(_) => {}
//             Err(e) => {
//                 println!("The error we get here is {}", e);
//             }
//         }
//     }

//     let mut handles = vec![];
//     let cp_users = users.clone();
//     users.into_iter().for_each(|name| {
//         for _ in 0..1 {
//             handles.push(tokio::spawn(spawn_front_post(
//                 Arc::clone(&name),
//                 back_addr.clone(),
//             )));
//         }
//     });

//     // get_backend_info(back_addr.clone(), true).await;

//     let (_, _, shut_sx) = backs.remove(2);
//     let _ = shut_sx.send(()).await;

//     // get_backend_info(back_addr.clone()).await;

//     let (_, _, shut_sx) = kps.remove(0);
//     let _ = shut_sx.send(()).await;
//     // get_backend_info(back_addr.clone(), true).await;
//     time::sleep(time::Duration::from_secs(20)).await;

//     // get_backend_info(back_addr.clone(), true).await;

//     for h in handles {
//         match h.await {
//             Ok(_) => {}
//             Err(e) => {
//                 println!("The error is {}", e)
//             }
//         }
//     }

//     assert_eq!(1, front.tribs(&cp_users[4]).await?.len());

//     shut_down_all(backs, kps).await;
//     Ok(())
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_keeper_shutdown_election() -> TribResult<()> {
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let mut kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr).await?;
//     let front = generate_client(back_addr.clone()).await?;

//     tokio::time::sleep(Duration::from_secs(3)).await;

//     get_backend_info(back_addr.clone(), true).await?;

//     // get_timestamp_info(back_addr.clone()).await?;

//     println!(
//         "\nstart keeper shutdown, keeper length: {}",
//         kps.len().to_string()
//     );
//     let (removed_addr, _, shut_sx) = kps.remove(0);
//     let _ = shut_sx.send(()).await;
//     println!(
//         "\nfinish shutdown, keeper length: {}",
//         kps.len().to_string()
//     );

//     tokio::time::sleep(Duration::from_secs(9)).await;

//     get_backend_info(back_addr.clone(), true).await?;

//     // get_timestamp_info(back_addr.clone()).await?;

//     return Ok(());
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_keeper_backend_shutdown_replication() -> TribResult<()> {
//     // backend shutdown after the keeper shutdown
//     // new leader should do the replication
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let mut kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr).await?;
//     let front = generate_client(back_addr.clone()).await?;

//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     tokio::time::sleep(Duration::from_secs(3)).await;

//     get_backend_info(back_addr.clone(), false).await?;
//     // get_timestamp_info(back_addr.clone()).await?;

//     let (removed_addr, _, shut_sx) = kps.remove(0);
//     let _ = shut_sx.send(()).await;
//     println!(
//         "\nfinish shutdown, keeper length: {}",
//         kps.len().to_string()
//     );

//     let (removed_addr, _, shut_sx) = backs.remove(3);
//     let _ = shut_sx.send(()).await;
//     println!(
//         "\nfinish shutdown, backs length: {}",
//         backs.len().to_string()
//     );

//     front.post("h8liu", "trib4", 4).await?;
//     front.post("fenglu", "trib5", 5).await?;
//     front.post("rkapoor", "trib6", 6).await?;

//     tokio::time::sleep(Duration::from_secs(15)).await;

//     get_backend_info(back_addr.clone(), false).await?;
//     // get_timestamp_info(back_addr.clone()).await?;

//     return Ok(());
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_backend_keeper_shutdown_replication() -> TribResult<()> {
//     // keeper shutdown after the backend shutdown
//     // new leader should do the replication
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let mut kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr).await?;
//     let front = generate_client(back_addr.clone()).await?;

//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "Hello, world.", 0).await?;
//     front.post("h8liu", "Just tribble it.", 2).await?;
//     front.post("fenglu", "Double tribble.", 0).await?;
//     front.post("rkapoor", "Triple tribble.", 0).await?;

//     tokio::time::sleep(Duration::from_secs(3)).await;

//     get_backend_info(back_addr.clone(), false).await?;

//     let (removed_addr_backs, _, shut_sx) = backs.remove(3);
//     let _ = shut_sx.send(()).await;
//     println!(
//         "\nfinish shutdown, backs length: {}",
//         backs.len().to_string()
//     );

//     let (removed_addr_kps, _, shut_sx) = kps.remove(0);
//     let _ = shut_sx.send(()).await;
//     println!(
//         "\nfinish shutdown, keeper length: {}",
//         kps.len().to_string()
//     );

//     front.post("h8liu", "trib4", 4).await?;
//     front.post("fenglu", "trib5", 5).await?;
//     front.post("rkapoor", "trib6", 6).await?;

//     tokio::time::sleep(Duration::from_secs(15)).await;

//     get_backend_info(back_addr.clone(), false).await?;

//     return Ok(());
// }

// #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
// async fn test_tribs_after_chaos() -> TribResult<()> {
//     // b3 shutdown
//     // k0 shutdown
//     // b3 revive
//     // b0 shutdown
//     // k0 revive
//     // k1 shutdown
//     let ((back_addr, used_back_addr, unused_back_addr), (kp_addr, used_kp_addr, unused_kp_addr)) =
//         generate_addr(6, 4, 4, 3);
//     let mut backs = set_up_backs(used_back_addr.clone()).await?;
//     let mut kps = set_up_kp(back_addr.clone(), kp_addr.clone(), used_kp_addr).await?;
//     let front = generate_client(back_addr.clone()).await?;

//     front.sign_up("h8liu").await?;
//     front.sign_up("fenglu").await?;
//     front.sign_up("rkapoor").await?;
//     front.post("h8liu", "trib0", 0).await?;
//     front.post("h8liu", "trib1", 2).await?;
//     front.post("fenglu", "trib2", 0).await?;
//     front.post("rkapoor", "trib3", 0).await?;

//     tokio::time::sleep(Duration::from_secs(3)).await;

//     get_backend_info(back_addr.clone(), false).await?;

//     let (removed_addr_backs, _, shut_sx) = backs.remove(3);
//     let _ = shut_sx.send(()).await;
//     println!(
//         "\nfinish shutdown, backs length: {}",
//         backs.len().to_string()
//     );

//     front.post("h8liu", "trib4", 4).await?;
//     front.post("fenglu", "trib5", 5).await?;
//     front.post("rkapoor", "trib6", 6).await?;

//     let (removed_addr_kps, _, shut_sx) = kps.remove(0);
//     let _ = shut_sx.send(()).await;
//     println!(
//         "\nfinish shutdown, keeper length: {}",
//         kps.len().to_string()
//     );

//     front.post("h8liu", "trib7", 7).await?;
//     front.post("fenglu", "trib8", 8).await?;
//     front.post("rkapoor", "trib9", 9).await?;

//     // setup b3
//     tokio::time::sleep(Duration::from_secs(30)).await;
//     let _ = set_up_backs(vec![removed_addr_backs]).await?;

//     front.post("h8liu", "trib10", 10).await?;
//     front.post("fenglu", "trib11", 11).await?;
//     front.post("rkapoor", "trib12", 12).await?;

//     tokio::time::sleep(Duration::from_secs(30)).await;
//     let (removed_addr_backs, _, shut_sx) = backs.remove(0);

//     front.post("h8liu", "trib13", 13).await?;
//     front.post("fenglu", "trib14", 14).await?;
//     front.post("rkapoor", "trib15", 15).await?;

//     let _ = set_up_kp(back_addr.clone(), kp_addr.clone(), vec![removed_addr_kps]).await?;

//     // shutdown k1
//     tokio::time::sleep(Duration::from_secs(60)).await;
//     let (removed_addr_kps, _, shut_sx) = kps.remove(0);
//     let _ = shut_sx.send(()).await;
//     println!(
//         "\nfinish shutdown, keeper length: {}",
//         kps.len().to_string()
//     );

//     tokio::time::sleep(Duration::from_secs(8)).await;

//     front.post("h8liu", "trib16", 16).await?;
//     front.post("fenglu", "trib17", 17).await?;
//     front.post("rkapoor", "trib18", 18).await?;

//     let mut tribs = front.tribs("h8liu").await?;
//     for i in 0..tribs.len() {
//         println!("{}", tribs[i].message);
//     }
//     println!("\n");

//     tribs = front.tribs("fenglu").await?;
//     for i in 0..tribs.len() {
//         println!("{}", tribs[i].message);
//     }
//     println!("\n");

//     tribs = front.tribs("rkapoor").await?;
//     for i in 0..tribs.len() {
//         println!("{}", tribs[i].message);
//     }
//     println!("\n");

//     get_backend_info(back_addr.clone(), true).await?;

//     return Ok(());
// }
