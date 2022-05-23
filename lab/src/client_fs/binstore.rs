use crate::client;
use crate::ops::{union, ListOp, LogOp, OpKind};
use async_trait::async_trait;
use log::info;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cmp;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::ParseIntError;
use std::sync::atomic;
use tribbler::colon;
use tribbler::err::{TribResult, TribblerError};
use tribbler::storage;
use tribbler::storage::{KeyValue, Storage};

pub struct BinStore {
    addrs: Vec<String>,
}

pub struct ReliableStore {
    addrs: Vec<String>,
    prefix: String,
    clock: atomic::AtomicU64,
    index: usize,
    simple: bool,
}

#[derive(Debug)]
pub enum KeyKind {
    KeyString,
    KeyList,
}

impl fmt::Display for KeyKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Sort operations by their clock.
fn sort_ops(ops: &mut Vec<LogOp>) {
    ops.sort_by(|a, b| {
        if a.clock < b.clock {
            cmp::Ordering::Less
        } else if a.clock > b.clock {
            cmp::Ordering::Greater
        } else {
            cmp::Ordering::Equal
        }
    });
}

// Returns self.prefix:kind:key.
pub fn compose_key(prefix: &str, kind: KeyKind, key: &str) -> String {
    format!(
        "{}:{:}:{}",
        colon::escape(prefix),
        colon::escape(kind.to_string()),
        colon::escape(key)
    )
}

// Decode a list of string into a list of ops.
fn decode_ops<T: DeserializeOwned>(ops: Vec<String>) -> TribResult<Vec<T>> {
    let mut result = vec![];
    for raw_op in ops {
        result.push(serde_json::from_str(&raw_op)?);
    }
    Ok(result)
}

// Encode a list of ops into a list of string.
fn encode_ops<T: Serialize>(ops: Vec<T>) -> TribResult<Vec<String>> {
    let mut result = vec![];
    for op in ops {
        result.push(serde_json::to_string(&op)?);
    }
    Ok(result)
}

fn box_err<T>(err: TribblerError) -> TribResult<T> {
    Err(Box::new(err))
}

// This is our backend interface.
impl ReliableStore {
    async fn get_store(&self, count: i32) -> TribResult<Box<dyn Storage>> {
        // start from the index and find the first alive one
        let mut idx = 0;
        let mut count = count;
        let mut limit = self.addrs.len() * 2;
        loop {
            limit -= 1;
            if limit == 0 {
                break box_err(TribblerError::Unknown(
                    "can't find any live store".to_string(),
                ));
            }
            if idx == self.addrs.len() {
                idx = 0;
            }
            let addr = &self.addrs[idx];
            let result = client::new_client(addr).await;
            match result {
                Ok(client) => {
                    let res = client.clock(0).await;
                    match res {
                        Ok(_) => {
                            count -= 1;
                            if count == 0 {
                                break Ok(client);
                            } else {
                                idx += 1;
                                continue;
                            }
                        }
                        Err(_) => {
                            // info!(
                            //     "Can't clock with store @ {}: {}",
                            //     self.addrs[idx],
                            //     err.to_string()
                            // );
                            idx += 1;
                            continue;
                        }
                    }
                }
                Err(err) => {
                    info!("Can't create client: {}", err.to_string());
                    idx += 1;
                    continue;
                }
            };
        }
    }

    // Gets the primary backend for key.
    async fn primary_store(&self) -> TribResult<Box<dyn Storage>> {
        self.get_store(1).await
    }

    async fn backup_store(&self) -> TribResult<Box<dyn Storage>> {
        self.get_store(2).await
    }

    async fn lookup(&self, key: &str) -> TribResult<u64> {
        loop {
            let primary = self.primary_store().await?;
            Ok(1)
        }
    }

    // Get sorted ops for key, key should be already composed.
    async fn get_sorted_ops(&self, key: &str) -> TribResult<Vec<LogOp>> {
        loop {
            let primary = self.primary_store().await?;
            let mut primary_log = if self.simple {
                match primary.get(key).await {
                    Err(err) => {
                        // info!("Error getting primary {:#?}", err);
                        continue;
                    }
                    Ok(Some(val)) => vec![val],
                    Ok(None) => vec![],
                }
            } else {
                match primary.list_get(key).await {
                    Err(err) => {
                        // info!("Error getting primary {:#?}", err);
                        continue;
                    }
                    Ok(ret) => ret.0,
                }
            };

            let backup = self.backup_store().await?;
            let mut backup_log = if self.simple {
                match backup.get(key).await {
                    Err(err) => {
                        // info!("Error getting backup {:#?}", err);
                        continue;
                    }
                    Ok(Some(val)) => vec![val],
                    Ok(None) => vec![],
                }
            } else {
                match backup.list_get(key).await {
                    Err(err) => {
                        // info!("Error getting backup {:#?}", err);
                        continue;
                    }
                    Ok(ret) => ret.0,
                }
            };
            // Union also removes all duplicates.
            union(&mut primary_log, &mut backup_log);
            let mut ops = decode_ops(primary_log)?;
            sort_ops(&mut ops);
            // info!("Read: {}:{}", &key, ops.len());
            return Ok(ops);
        }
    }

    // Returns self.prefix:kind:key.
    fn compose_key(&self, kind: KeyKind, key: &str) -> String {
        compose_key(&self.prefix, kind, key)
    }

    // Sync my clock to at least at_least, if increment is true,
    // increment my clock to at least at_least. Return the new clock.
    fn clock(&self, at_least: u64, increment: bool) -> TribResult<()> {
        let my_clock = self.clock.load(atomic::Ordering::SeqCst);
        if my_clock == u64::MAX {
            return Err(Box::new(TribblerError::MaxedSeq));
        }
        let prev_clock = self
            .clock
            .fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst, |v| {
                if v < at_least {
                    Some(at_least)
                } else {
                    if increment {
                        Some(v)
                    } else {
                        Some(v + 1)
                    }
                }
            })
            .unwrap();
        Ok(())
    }

    fn get_clock(&self) -> u64 {
        self.clock.load(atomic::Ordering::SeqCst)
    }

    fn incr_clock(&self) -> u64 {
        self.clock
            .fetch_update(atomic::Ordering::SeqCst, atomic::Ordering::SeqCst, |v| {
                Some(v + 1)
            })
            .unwrap()
    }

    async fn get_op_clock(
        &self,
        primary: &Box<dyn Storage>,
        backup: &Box<dyn Storage>,
    ) -> TribResult<u64> {
        let mut op_clock = primary.clock(0).await?;
        op_clock = backup.clock(op_clock).await?;
        Ok(op_clock)
    }

    async fn append_to<'a>(
        &'a self,
        store: Box<dyn Storage>,
        kv: &'a storage::KeyValue,
        clock: u64,
        kind: OpKind,
    ) -> TribResult<()> {
        // info!("Write: clock {}, {}:{}", clock, &kv.key, &JV.value);
        let key_kind = match &kind {
            OpKind::KeyList(_) => KeyKind::KeyList,
            OpKind::KeyString => KeyKind::KeyString,
        };

        let op = LogOp {
            clock,
            val: kv.value.clone(),
            kind,
        };

        let mykv = KeyValue {
            key: self.compose_key(key_kind, &kv.key),
            value: serde_json::to_string(&op)?,
        };
        let success = if self.simple {
            store.set(&mykv).await?
        } else {
            store.list_append(&mykv).await?
        };
        if success {
            Ok(())
        } else {
            box_err(TribblerError::Unknown(
                "list_append returned false".to_string(),
            ))
        }
    }
}

#[async_trait]
impl storage::KeyString for ReliableStore {
    async fn get(&self, key: &str) -> TribResult<Option<String>> {
        let ops: Vec<LogOp> = self
            .get_sorted_ops(&self.compose_key(KeyKind::KeyString, key))
            .await?;
        if ops.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(ops.last().unwrap().val.clone()))
        }
    }

    async fn set(&self, kv: &storage::KeyValue) -> TribResult<bool> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            let op_clock = self.get_op_clock(&primary, &backup).await?;
            match self
                .append_to(primary, kv, op_clock, OpKind::KeyString)
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match self
                .append_to(backup, kv, op_clock, OpKind::KeyString)
                .await
            {
                Err(_) => continue,
                Ok(_) => break,
            }
        }
        Ok(true)
    }

    async fn keys(&self, p: &storage::Pattern) -> TribResult<storage::List> {
        loop {
            let primary = self.primary_store().await?;
            match primary.keys(p).await {
                Err(_) => continue,
                Ok(result) => return Ok(result),
            }
        }
    }
}

#[async_trait]
impl storage::KeyList for ReliableStore {
    async fn list_get(&self, key: &str) -> TribResult<storage::List> {
        let ops: Vec<LogOp> = self
            .get_sorted_ops(&self.compose_key(KeyKind::KeyList, key))
            .await?;
        let mut result = vec![];
        for op in ops {
            match op.kind {
                OpKind::KeyList(ListOp::Append) => result.push(op.val),
                OpKind::KeyList(ListOp::Remove) => result.retain(|x| x != &op.val),
                OpKind::KeyList(ListOp::Clear) => result = vec![],
                OpKind::KeyString => {
                    return box_err(TribblerError::Unknown(
                        "OpKind shouln't be KeyString".to_string(),
                    ))
                }
            }
        }
        Ok(storage::List(result))
    }

    async fn list_keys(&self, p: &storage::Pattern) -> TribResult<storage::List> {
        loop {
            let primary = self.primary_store().await?;
            match primary.list_keys(p).await {
                Err(_) => continue,
                Ok(result) => return Ok(result),
            }
        }
    }

    async fn list_append(&self, kv: &storage::KeyValue) -> TribResult<bool> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            let op_clock = self.get_op_clock(&primary, &backup).await?;
            match self
                .append_to(primary, kv, op_clock, OpKind::KeyList(ListOp::Append))
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match self
                .append_to(backup, kv, op_clock, OpKind::KeyList(ListOp::Append))
                .await
            {
                Err(_) => continue,
                Ok(_) => break,
            }
        }
        Ok(true)
    }

    async fn list_remove(&self, kv: &storage::KeyValue) -> TribResult<u32> {
        let prev_list = self.list_get(&kv.key).await?.0;
        let count = prev_list.iter().filter(|&x| x == &kv.value).count();
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            let op_clock = self.get_op_clock(&primary, &backup).await?;
            match self
                .append_to(primary, kv, op_clock, OpKind::KeyList(ListOp::Remove))
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match self
                .append_to(backup, kv, op_clock, OpKind::KeyList(ListOp::Remove))
                .await
            {
                Err(_) => continue,
                Ok(_) => break,
            }
        }
        Ok(count as u32)
    }
}

#[async_trait]
impl storage::Storage for ReliableStore {
    async fn clock(&self, at_least: u64) -> TribResult<u64> {
        loop {
            let primary = self.primary_store().await?;
            match primary.clock(at_least).await {
                Err(_) => continue,
                Ok(clk) => return Ok(clk),
            }
        }
    }
}

impl BinStore {
    pub fn new(addrs: Vec<String>) -> BinStore {
        BinStore { addrs }
    }
}

pub fn hash_name_to_idx(name: &str, len: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish() as usize % len
}

#[async_trait]
impl storage::BinStorage for BinStore {
    async fn bin(&self, name: &str) -> TribResult<Box<dyn Storage>> {
        let length = self.addrs.len();
        let idx: usize = hash_name_to_idx(name, length);
        // info!("Create bin for {} -> {}", &name, idx);
        let mut new_addrs = Vec::new();
        for i in 0..length {
            new_addrs.push(self.addrs[(i + idx) % length].clone());
        }
        Ok(Box::new(ReliableStore {
            prefix: name.to_string(),
            index: idx,
            addrs: new_addrs,
            clock: atomic::AtomicU64::new(0),
            simple: false,
        }))
    }
}

impl BinStore {
    pub async fn keeper_bin(&self, name: &str) -> TribResult<Box<dyn Storage>> {
        let length = self.addrs.len();
        let idx: usize = hash_name_to_idx(name, length);
        let mut new_addrs = Vec::new();

        for i in 0..length {
            new_addrs.push(self.addrs[(i + idx) % length].clone());
        }
        Ok(Box::new(ReliableStore {
            prefix: name.to_string(),
            index: idx,
            addrs: new_addrs,
            clock: atomic::AtomicU64::new(0),
            simple: true,
        }))
    }
}

#[async_trait]
impl ServerFileSystem for ReliableStore{
    async fn read(
        &self,
        _req: &Request,
        inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
    ) -> TritonFileResult<String>{
        loop {
            let primary = self.primary_store().await?;
            match primary.read(_req, inode, fh, offset, size, _flags, _lock_owner).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn write(
        &mut self,
        _req: &Request,
        inode: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        #[allow(unused_variables)] flags: i32,
        _lock_owner: Option<u64>,
    ) -> TritonFileResult<u32>{
        loop {
            let primary = self.primary_store().await?;
            match primary.write(_req, inode, fh, offset, data, _write_flags, flags, _lock_owner)
            .await{
                Err(_) => continue,
                Ok(_) => break,
            }
        }

        loop{
            let backup = self.backup_store().await?;
            match backup.write(_req, inode, fh, offset, data, _write_flags, flags, _lock_owner)
            .await{
                Err(_) => continue,
                Ok(res) => return Ok(res)
            }
        }
    }

    async fn lookup(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<FileAttr>{
        loop {
            let primary = self.primary_store().await?;
            match primary.lookup(req, parent, name).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn unlink(&mut self, req: &Request, parent: u64, name: &OsStr) -> TritonFileResult<()>{
        loop {
            let primary = self.primary_store().await?;
            match primary.unlink(req, parent, name)
            .await{
                Err(_) => continue,
                Ok(_) => break,
            }
        }

        loop{
            let backup = self.backup_store().await?;
            match backup.unlink(req, parent, name)
            .await{
                Err(_) => continue,
                Ok(_) => return Ok()
            }
        }
    }

    async fn create(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
        mut mode: u32,
        _umask: u32,
        flags: i32,
    ) -> TritonFileResult<(FileAttr, u64)>{
        loop {
            let primary = self.primary_store().await?;
            match primary.create(req, parent, name)
            .await{
                Err(_) => continue,
                Ok(_) => break,
            }
        }

        loop{
            let backup = self.backup_store().await?;
            match backup.create(req, parent, name)
            .await{
                Err(_) => continue,
                Ok(res) => return Ok(res)
            }
        }
    }
}
