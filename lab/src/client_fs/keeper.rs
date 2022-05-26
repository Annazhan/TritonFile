use std::{sync::{Arc, atomic}, vec};

use log::info;
use serde::{Serialize, Deserialize};
use tokio::{sync::{Mutex, mpsc::{Receiver, error::TryRecvError}}, time::{self, timeout}};
use tribbler::{storage::{self, Storage}, config::KeeperConfig, error::{TritonFileResult}, storage::KeyValue};
use core::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;

use crate::client_fs::client::new_client;

use super::binstore::{self, BinStore, hash_name_to_idx};

#[derive(Serialize, Deserialize, Debug)]
enum LiveState {
    True(bool),
    False(bool),
}

const KEY_KEEPER: &str = "KEEPER";
const KEY_TIMESTAMP: &str = "TIMESTAMP";
const KEY_KEEPER_REPLICATE: &str = "LIVE_LIST_STATE";

struct Keeper{
    clock: Arc<AtomicU64>,
    // Addresses for keepers.
    addrs: Arc<Mutex<Vec<String>>>,
    // Addresses for backends.
    backs: Arc<Mutex<Vec<String>>>,
    this: usize,
    keep_bin: Box<dyn storage::Storage>,
    timestamp: Arc<AtomicU64>,
    live_list: Arc<Mutex<Vec<bool>>>,
}

impl Keeper {
    async fn new(kp_addrs: Vec<String>, bk_addrs: Vec<String>, ready:Option<Sender<bool>>,  storage: Box<dyn Storage>) -> Keeper{
        Keeper{
            clock: Arc::new(atomic::AtomicU64::new(1)),
            addrs: Arc::new(Mutex::new(kp_addrs.clone())),
            backs: Arc::new(Mutex::new(bk_addrs.clone())),
            this: 0,
            keep_bin: storage,
            timestamp: Arc::new(atomic::AtomicU64::new(1)), 
            live_list: Arc::new(Mutex::new(vec![false; bk_addrs.len()])),
        }
    }

    
}

fn send_signal(chan: &Option<Sender<bool>>, signal: bool) -> TritonFileResult<()> {
    match chan {
        Some(sender) => match sender.send(signal) {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        },
        None => Ok(()),
    }
}

fn should_shutdown(shutdown: &mut Option<Receiver<()>>) -> TritonFileResult<bool> {
    match shutdown {
        Some(recver) => match recver.try_recv() {
            Ok(_) => Ok(true),
            Err(e) => match e {
                TryRecvError::Disconnected => return Err(Box::new(e)),
                TryRecvError::Empty => Ok(false),
            },
        },
        None => Ok(false),
    }
}

pub async fn serve_keeper(kc: KeeperConfig) -> TritonFileResult<()> {
    info!("Serve_keeper request, config: {:?}", &kc);
    let backs = kc.backs.clone();
    let keep_bin = BinStore::new(backs).keeper_bin(KEY_KEEPER).await?;
    let keeper = Arc::new(Keeper::new(kc.addrs, kc.backs, kc.ready, keep_bin).await);
    let mut shutdown = kc.shutdown;

    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    let _ = tokio::spawn(background_update_clock(Arc::clone(&keeper), receiver));
    loop {
        if should_shutdown(&mut shutdown)? {
            let _ = sender.send(());
            return Ok(());
        }

        // 3 seconds between each round.
        let wait = time::sleep_until(time::Instant::now() + time::Duration::from_secs(3));
        match keeper.serve_one_round().await {
            Err(err) => {
                info!("Encountered error: {}", err.to_string());
                continue;
            }
            Ok(_) => wait.await,
        }
    }
}

async fn background_update_clock(keeper: Arc<Keeper>, shutdown: Receiver<()>) -> TritonFileResult<()> {
    let mut shutdown = Some(shutdown);
    loop {
        if should_shutdown(&mut shutdown)? {
            return Ok(());
        }
        let wait = time::sleep_until(time::Instant::now() + time::Duration::from_secs(1));
        match keeper.update_keeper_clock().await {
            Err(err) => info!("Update clock error: {}", err.to_string()),
            Ok(_) => (),
        }
        wait.await;
    }
}

fn next_live(this: usize, live_list: &Vec<bool>, step: i32) -> Option<usize> {
    let mut this = this;
    // Only check for one round.
    for _ in 0..live_list.len() - 1 {
        // Increment/decrement and wrap around.
        if step > 0 {
            if this == live_list.len() - 1 {
                this = 0;
            } else {
                this += 1;
            }
        } else {
            if this == 0 {
                this = live_list.len() - 1;
            } else {
                this -= 1;
            }
        }
        if live_list[this] {
            return Some(this);
        }
    }
    // We went one round and there is no other, return None.
    None
}

fn key_primary_idx(key_id: usize, live_list: &Vec<bool>) -> Option<usize> {
    let mut this = key_id;
    // Only check for one round.
    for _ in 0..live_list.len() - 1 {
        // If the key is right on the live server
        if live_list[this] {
            return Some(this);
        }
        if this == live_list.len() - 1 {
            this = 0;
        } else {
            this += 1;
        }
    }
    // We went one round and there is no other, return None.
    None
}

impl Keeper {
    fn print_name(&self) -> String {
        format!(
            "Keeper {}({})",
            self.this,
            self.timestamp.load(atomic::Ordering::SeqCst)
        )
    }

    async fn serve_one_round(&self) -> TritonFileResult<()> {
        info!("{} serving one round", self.print_name(),);
        let living_keepers = self.living_keepers().await?;
            // I'm leader, serve and return.
        info!("{}: I'm leader", self.print_name());
        let mut old_live_list = self.live_list.lock().await;
            // Get new live list, save it, and serve as keeper.
        let new_live_list = self.broadcast(time::Duration::from_millis(1000)).await?;
        self.serve_as_leader(&old_live_list, &new_live_list).await?;
        // It is important that we save the live list AFTER
        // doing the work. So that we don't miss the work if
        // we crash mid way.
        self.save_live_list_backup(&new_live_list).await?;
        *old_live_list = new_live_list;
        //return Ok(());
        // If we reach here, we might become the leader.
        let min_living_keeper = living_keepers.iter().min().unwrap();
        if *min_living_keeper == self.this {
            info!("{}: leader dead, I'm the new leader", self.print_name());
            // When we become leader, inherit live_list from last
            // leader, or use default (all dead).
            let last_leader_live_list = match self.get_live_list_backup().await? {
                None => vec![false; self.backs.lock().await.len()],
                Some(list) => list,
            };
            let mut keeper_live_list = self.live_list.lock().await;
            *keeper_live_list = last_leader_live_list;
        } else {
            info!("{}: leader dead, but I'm not min idx", self.print_name());
        }
        Ok(())
    }

    async fn hash_name_to_idx(&self, name: &str) -> usize {
        let length = { self.backs.lock().await.len() };
        hash_name_to_idx(name, length)
    }

    // Serve a round in loop as a leader. When we return, the old live
    // list is replaced in-place by the new live list.
    async fn serve_as_leader(
        &self,
        old_live_list: &Vec<bool>,
        new_live_list: &Vec<bool>,
    ) -> TritonFileResult<()> {
        info!("{} serving as leader", self.print_name());
        // This should return in at most 1+e seconds.
        for i in 0..new_live_list.len() {
            if new_live_list[i] != old_live_list[i] {
                self.manage_replicate(&new_live_list, old_live_list).await?;
                // let _ = self.set_replicate_state(&Vec::new());
                break;
            }
        }
        Ok(())
    }


    async fn replicate(
        &self,
        from: usize,
        to: usize,
        for_addr: usize,
        live_list: &Vec<bool>,
    ) -> TritonFileResult<()> {
        info!("{}: replicating {} to {}", self.print_name(), from, to);
        // get all keys from the 'from'
        let p = storage::Pattern {
            prefix: "".to_string(),
            suffix: "".to_string(),
        };

        let (from_cli, to_cli) = {
            let backs = self.backs.lock().await;
            (
                new_client(&backs[from]).await?,
                new_client(&backs[to]).await?,
            )
        };

        let keys = from_cli.list_keys(&p).await?.0;

        for key in keys {
            // Key has the shape of username:kind:key.
            let key_name = key.split(':').collect::<Vec<&str>>()[0];
            let idx = self.hash_name_to_idx(key_name).await;
            // if the hash key is right on the primary
            let primary_for_key = key_primary_idx(idx, live_list).unwrap();

            if primary_for_key == for_addr {
                let values_in_from = from_cli.list_get(&key.as_str()).await?.0;
                let values_in_to = to_cli.list_get(&key.as_str()).await?.0;
                for val in values_in_from {
                    if !values_in_to.contains(&val) {
                        let kv = storage::KeyValue {
                            key: key.clone(),
                            value: val.clone(),
                        };
                        to_cli.list_append(&kv).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn manage_replicate(
        &self,
        new_live_list: &Vec<bool>,
        old_live_list: &Vec<bool>,
    ) -> TritonFileResult<()> {
        // Only replicate if there are at least 3 backends alive.
        if new_live_list.iter().filter(|&x| *x).count() >= 3 {
            for idx in 0..new_live_list.len() {
                if !old_live_list[idx] && new_live_list[idx] {
                    info!("{}: back {} joined", self.print_name(), idx);
                    // Join. We assume there are always three machines.
                    let before = next_live(idx, new_live_list, -1).unwrap();
                    let after = next_live(idx, new_live_list, 1).unwrap();
                    self.handle_join(before, after, idx, new_live_list).await?;
                } else if old_live_list[idx] && !new_live_list[idx] {
                    info!("{}: back {} left", self.print_name(), idx);
                    // Leave.
                    let before = next_live(idx, new_live_list, -1).unwrap();
                    let after = next_live(idx, new_live_list, 1).unwrap();
                    let after_after = next_live(after, new_live_list, 1).unwrap();
                    self.handle_leave(before, after, after_after, new_live_list)
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn handle_join(
        &self,
        primary: usize,
        old_backup: usize,
        new_backup: usize,
        live_list: &Vec<bool>,
    ) -> TritonFileResult<()> {
        self.replicate(old_backup, new_backup, new_backup, live_list)
            .await?;
        self.replicate(primary, new_backup, primary, live_list)
            .await?;
        // self.replicate(old_backup, new_backup, primary, live_list)
        //     .await?;

        Ok(())
    }

    async fn handle_leave(
        &self,
        before: usize,
        after: usize,
        after_after: usize,
        live_list: &Vec<bool>,
    ) -> TritonFileResult<()> {
        self.replicate(after, after_after, after, live_list).await?;
        self.replicate(before, after, before, live_list).await?;
        Ok(())
    }

    // Broadcast to sync the clock and return a live list.
    async fn broadcast(&self, time_limit: time::Duration) -> TritonFileResult<Vec<bool>> {
        info!("Broadcast request");
        let backs = self.backs.lock().await;
        let mut handles = vec![];
        let mut live_list = vec![false; backs.len()];
        for idx in 0..backs.len() {
            match new_client(&backs[idx].clone()).await {
                Ok(c) => {
                    let clock = Arc::clone(&self.clock);
                    let client = Arc::new(c);
                    let handle = tokio::spawn(sync_clock_with(client, time_limit, clock, idx));
                    handles.push(handle);
                }
                Err(_) => (),
            }
        }

        // Every thread should terminate in time_limit, so there is no
        // deadlock.
        for handle in handles {
            match handle.await {
                Ok(ret) => {
                    if let Ok(idx) = ret {
                        live_list[idx] = true
                    }
                }
                // Thread panic, throw.
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(live_list)
    }

    async fn update_keeper_clock(&self) -> TritonFileResult<u64> {
        let keep_bin = &self.keep_bin;
        let mut max_time = self.timestamp.load(atomic::Ordering::SeqCst);

        let keeper_clocks: Vec<u64> = self.collect_keeper_clock().await?;
        for i in 0..keeper_clocks.len() {
            if keeper_clocks[i] > max_time {
                max_time = keeper_clocks[i];
            }
        }

        max_time = max_time + 1;
        let key = self.compose_timestamp_key(self.this);
        let kv = KeyValue {
            key,
            value: max_time.to_string(),
        };
        keep_bin.set(&kv).await?;
        let _ =
            self.timestamp
                .fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst, |v| {
                    if max_time > v {
                        Some(max_time)
                    } else {
                        Some(v)
                    }
                });

        Ok(max_time)
    }


    async fn living_keepers(&self) -> TritonFileResult<Vec<usize>> {
        let clocks = self.collect_keeper_clock().await?;
        // Basically 6 seconds, because we update timestamp in every
        // second.
        let timeout = 6;
        let my_clock = self.timestamp.load(atomic::Ordering::SeqCst);
        let mut living_keepers = vec![];
        for idx in 0..clocks.len() {
            if idx == self.this {
                living_keepers.push(idx);
                continue;
            }
            if my_clock < clocks[idx] || my_clock - clocks[idx] < timeout {
                living_keepers.push(idx);
            }
        }
        info!(
            "{}: living keepers are {:?}",
            self.print_name(),
            &living_keepers
        );
        Ok(living_keepers)
    }

    fn compose_timestamp_key(&self, idx: usize) -> String {
        format!("{}_{}", KEY_TIMESTAMP, idx)
    }

    fn compose_replicate_state_key(&self) -> String {
        return KEY_KEEPER_REPLICATE.to_string();
    }

    async fn collect_keeper_clock(&self) -> TritonFileResult<Vec<u64>> {
        let mut clocks = vec![];
        let addrs = { self.addrs.lock().await.clone() };
        let keep_bin = &self.keep_bin;
        for idx in 0..addrs.len() {
            if self.this == idx {
                clocks.push(self.timestamp.load(atomic::Ordering::SeqCst));
            } else {
                let key = self.compose_timestamp_key(idx);
                if let Some(clock_str) = keep_bin.get(&key).await? {
                    let clock: u64 = clock_str.parse().unwrap();
                    clocks.push(clock);
                } else {
                    clocks.push(0);
                }
            }
        }
        // info!("keeper clocks: {:?}", &clocks);
        Ok(clocks)
    }

    async fn save_live_list_backup(&self, live_list: &Vec<bool>) -> TritonFileResult<bool> {
        let keep_bin = &self.keep_bin;
        let kv = KeyValue {
            key: self.compose_replicate_state_key(),
            value: serde_json::to_string(&live_list.clone()).unwrap(),
        };
        keep_bin.set(&kv).await
    }

    async fn get_live_list_backup(&self) -> TritonFileResult<Option<Vec<bool>>> {
        let keep_bin = &self.keep_bin;
        match keep_bin.get(&self.compose_replicate_state_key()).await? {
            None => Ok(None),
            Some(ret) => Ok(Some(serde_json::from_str(&ret).unwrap())),
        }
    }
}




// Send a clock() request to client to sync up the clock, if
// doesn't receive response in wait_for, simply give up. If
// network error occurs, return it.
async fn sync_clock_with(
    client: Arc<Box<dyn storage::Storage>>,
    wait_for: time::Duration,
    clock: Arc<atomic::AtomicU64>,
    idx: usize,
) -> TritonFileResult<usize> {
    let time = clock.load(atomic::Ordering::SeqCst);
    match timeout(wait_for, client.clock(time)).await {
        Ok(time) => {
            // We don't handle network error.
            let t = time?;
            let _ = clock.fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst, |v| {
                if v < t {
                    Some(t)
                } else {
                    None
                }
            });
        }
        // We get here if timeout is triggered before result comes
        // in. In that case just give up.
        Err(_) => {
            info!("Clock sync timed out");
        }
    };
    Ok(idx)
}
