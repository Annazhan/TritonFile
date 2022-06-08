use super::ops::{union, ListOp, LogOp, OpKind};
use async_trait::async_trait;
use fuser::FileAttr;
use fuser::TimeOrNow;
use libc::c_int;
use log::info;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tribbler::storage::ContentList;
use tribbler::storage::InodeList;
use std::cmp;
use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::fmt;
use fuser::FileType;
use std::hash::{Hash, Hasher};
use std::sync::atomic;
use std::time::SystemTime;
use tribbler::colon;
use tribbler::error::{TritonFileError, TritonFileResult};
use tribbler::storage;
use tribbler::storage::{FileRequest, KeyValue, ServerFileSystem, Storage};
use crate::client_fs::binstore::storage::DataList;

use super::client;

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
fn decode_ops<T: DeserializeOwned>(ops: Vec<String>) -> TritonFileResult<Vec<T>> {
    let mut result = vec![];
    for raw_op in ops {
        result.push(serde_json::from_str(&raw_op)?);
    }
    Ok(result)
}

// Encode a list of ops into a list of string.
fn encode_ops<T: Serialize>(ops: Vec<T>) -> TritonFileResult<Vec<String>> {
    let mut result = vec![];
    for op in ops {
        result.push(serde_json::to_string(&op)?);
    }
    Ok(result)
}

fn box_err<T>(err: TritonFileError) -> TritonFileResult<T> {
    Err(Box::new(err))
}

// This is our backend interface.
impl ReliableStore {
    async fn get_store(&self, count: i32) -> TritonFileResult<Box<dyn Storage>> {
        // start from the index and find the first alive one
        let mut idx = 0;
        let mut count = count;
        info!("{} addrs len: {}", count, self.addrs.len());
        let mut limit = self.addrs.len() * 2;
        loop {
            limit -= 1;
            if limit == 0 {
                break box_err(TritonFileError::Unknown(
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
                        Err(err) => {
                            info!(
                                "Can't clock with store @ {}: {}",
                                self.addrs[idx],
                                err.to_string()
                            );
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
    async fn primary_store(&self) -> TritonFileResult<Box<dyn Storage>> {
        self.get_store(1).await
    }

    async fn backup_store(&self) -> TritonFileResult<Box<dyn Storage>> {
        self.get_store(2).await
    }

    // async fn lookup(&self, key: &str) -> TritonFileResult<u64> {
    //     loop {
    //         let primary = self.primary_store().await?;
    //         Ok(1)
    //     }
    // }

    // Get sorted ops for key, key should be already composed.
    async fn get_sorted_ops(&self, key: &str) -> TritonFileResult<Vec<LogOp>> {
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
    fn clock(&self, at_least: u64, increment: bool) -> TritonFileResult<()> {
        let my_clock = self.clock.load(atomic::Ordering::SeqCst);
        if my_clock == u64::MAX {
            return Err(Box::new(TritonFileError::MaxedSeq));
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
    ) -> TritonFileResult<u64> {
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
    ) -> TritonFileResult<()> {
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
            box_err(TritonFileError::Unknown(
                "list_append returned false".to_string(),
            ))
        }
    }
}

#[async_trait]
impl storage::KeyString for ReliableStore {
    async fn get(&self, key: &str) -> TritonFileResult<Option<String>> {
        let ops: Vec<LogOp> = self
            .get_sorted_ops(&self.compose_key(KeyKind::KeyString, key))
            .await?;
        if ops.len() == 0 {
            Ok(None)
        } else {
            Ok(Some(ops.last().unwrap().val.clone()))
        }
    }

    async fn set(&self, kv: &storage::KeyValue) -> TritonFileResult<bool> {
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

    async fn keys(&self, p: &storage::Pattern) -> TritonFileResult<storage::List> {
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
    async fn list_get(&self, key: &str) -> TritonFileResult<storage::List> {
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
                    return box_err(TritonFileError::Unknown(
                        "OpKind shouln't be KeyString".to_string(),
                    ))
                }
            }
        }
        Ok(storage::List(result))
    }

    async fn list_keys(&self, p: &storage::Pattern) -> TritonFileResult<storage::List> {
        loop {
            let primary = self.primary_store().await?;
            match primary.list_keys(p).await {
                Err(_) => continue,
                Ok(result) => return Ok(result),
            }
        }
    }

    async fn list_append(&self, kv: &storage::KeyValue) -> TritonFileResult<bool> {
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

    async fn list_remove(&self, kv: &storage::KeyValue) -> TritonFileResult<u32> {
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
    async fn clock(&self, at_least: u64) -> TritonFileResult<u64> {
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
    async fn bin(&self, name: &str) -> TritonFileResult<Box<dyn Storage>> {
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
    pub async fn keeper_bin(&self, name: &str) -> TritonFileResult<Box<dyn Storage>> {
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
impl ServerFileSystem for ReliableStore {
    async fn get_all_nodes(
        &self,
        for_addr: usize,
        len: usize,
    ) -> TritonFileResult<Option<(InodeList, ContentList)>>{
        Ok(None)
    }

    async fn write_all_nodes(
        &self,
        inode_list: InodeList,
        content_list: ContentList,
    ) -> TritonFileResult<()>{
        Ok(())
    }

    async fn init(&self, _req: &FileRequest) -> TritonFileResult<c_int>{
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;

            match primary.init(_req).await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup.init(_req).await
            {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn read(
        &self,
        _req: &FileRequest,
        inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
    ) -> TritonFileResult<(Option<String>, c_int)> {
        loop {
            let primary = self.primary_store().await?;
            match primary
                .read(_req, inode, fh, offset, size, _flags, _lock_owner)
                .await
            {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn write(
        &self,
        _req: &FileRequest,
        inode: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        #[allow(unused_variables)] flags: i32,
        _lock_owner: Option<u64>,
    ) -> TritonFileResult<(Option<u32>, c_int)> {
        info!("call bin storage write() {}", inode);
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;

            info!("primary call write() {}", inode);
            match primary
                .write(
                    _req,
                    inode,
                    fh,
                    offset,
                    data,
                    _write_flags,
                    flags,
                    _lock_owner,
                )
                .await
            {
                Err(_) => {
                    info!("binstorage prime write fail");
                    continue
                },
                Ok(_) => (),
            }

            info!("backup call write() {}", inode);
            match backup
                .write(
                    _req,
                    inode,
                    fh,
                    offset,
                    data,
                    _write_flags,
                    flags,
                    _lock_owner,
                )
                .await
            {
                Err(_) => {
                    info!("binstorage backup write fail");
                    continue},
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn lookup(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)> {
        loop {
            let primary = self.primary_store().await?;
            match primary.lookup(req, parent, name).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn unlink(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<c_int> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary.unlink(req, parent, name).await {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup.unlink(req, parent, name).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn create(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
        mut mode: u32,
        _umask: u32,
        flags: i32,
    ) -> TritonFileResult<(Option<(FileAttr, u64)>, c_int)> {
        info!("At binstorage create");
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;

            match primary.create(req, parent, name, mode, _umask, flags).await {
                Err(e) => {
                    info!{"{}", e};
                    info!("binstorage primary create has error");
                    continue},
                Ok(_) => (),
            }
            match backup.create(req, parent, name, mode, _umask, flags).await {
                Err(e) => {
                    info!{"{}", e};
                    info!("binstorage backup create has error");
                    continue},
                Ok(res) => return Ok(res),
            }
        }
    }
    async fn getattr(
        &self,
        _req: &FileRequest,
        ino: u64,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)> {
        loop {
            info!("bin storage getattr");
            let primary = self.primary_store().await?;
            match primary.getattr(_req, ino).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn open(
        &self,
        _req: &FileRequest,
        _ino: u64,
        _flags: i32,
    ) -> TritonFileResult<(Option<(u64, u32)>, c_int)> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary.open(_req, _ino, _flags).await {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup.open(_req, _ino, _flags).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn release(
        &self,
        _req: &FileRequest,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
    ) -> TritonFileResult<c_int> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary
                .release(_req, _ino, _fh, _flags, _lock_owner, _flush)
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup
                .release(_req, _ino, _fh, _flags, _lock_owner, _flush)
                .await
            {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn setxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        name: &OsStr,
        _value: &[u8],
        flags: i32,
        position: u32,
    ) -> TritonFileResult<(c_int)> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary
                .setxattr(_req, ino, name, _value, flags, position)
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup
                .setxattr(_req, ino, name, _value, flags, position)
                .await
            {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    //reply Vec<u8> as string
    async fn getxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        name: &OsStr,
        size: u32,
    ) -> TritonFileResult<(Option<(String, u32)>, c_int)> {
        loop {
            let primary = self.primary_store().await?;
            match primary.getxattr(_req, ino, name, size).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn listxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        size: u32,
    ) -> TritonFileResult<(Option<(String, u32)>, c_int)> {
        loop {
            let primary = self.primary_store().await?;
            match primary.listxattr(_req, ino, size).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn access(&self, _req: &FileRequest, ino: u64, mask: i32) -> TritonFileResult<(c_int)> {
        loop {
            let primary = self.primary_store().await?;
            match primary.access(_req, ino, mask).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn rename(
        &self,
        _req: &FileRequest,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        flags: u32,
    ) -> TritonFileResult<c_int> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary
                .rename(_req, parent, name, newparent, newname, flags)
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup
                .rename(_req, parent, name, newparent, newname, flags)
                .await
            {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn setattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        _mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        flags: Option<u32>,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary
                .setattr(
                    _req, ino, mode, uid, gid, size, _atime, _mtime, _ctime, fh, _crtime, _chgtime,
                    _bkuptime, flags,
                )
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup
                .setattr(
                    _req, ino, mode, uid, gid, size, _atime, _mtime, _ctime, fh, _crtime, _chgtime,
                    _bkuptime, flags,
                )
                .await
            {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn opendir(
        &self,
        req: &FileRequest,
        inode: u64,
        flags: i32,
    ) -> TritonFileResult<(Option<(u64, u32)>, c_int)>{
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary.opendir(req, inode, flags).await {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup.opendir(req, inode, flags).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn readdir(
        &self,
        _req: &FileRequest,
        inode: u64,
        _fh: u64,
        offset: i64,
    ) -> TritonFileResult<(Option<(u64, i64, FileType, DataList)>, c_int)>{
        loop {
            let primary = self.primary_store().await?;
            match primary.readdir(_req, inode, _fh, offset).await {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn releasedir(
        &self,
        _req: &FileRequest,
        inode: u64,
        _fh: u64,
        _flags: i32,
    ) -> TritonFileResult<c_int> {
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary
                .releasedir(_req, inode, _fh, _flags)
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup
                .releasedir(_req, inode, _fh, _flags)
                .await
            {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }

    async fn mkdir(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
        mut mode: u32,
        _umask: u32,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)>{
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            match primary
                .mkdir(req, parent, name, mode, _umask)
                .await
            {
                Err(_) => continue,
                Ok(_) => (),
            }
            match backup
            .mkdir(req, parent, name, mode, _umask)
            .await
            {
                Err(_) => continue,
                Ok(res) => return Ok(res),
            }
        }
    }
}
