// use crate::lab3::client;
// use crate::lab3::ops::{union, ListOp, LogOp, OpKind};
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

impl ReliableStore {
    async fn get_store(&self, count: i32) -> TritonFileResult<Box<dyn Storage>> {
        // start from the index and find the first alive one
        let mut idx = 0;
        let mut count = count;
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
    async fn primary_store(&self) -> TritonFileResult<Box<dyn Storage>> {
        self.get_store(1).await
    }

    async fn backup_store(&self) -> TritonFileResult<Box<dyn Storage>> {
        self.get_store(2).await
    }

    async fn lookup(&self, req:u64, parent:u64, name:&OsStr) -> TritonFileResult<>{
        loop {
            let primary = self.primary_store().await?;
            match primary.lookup(req, parent, name).await {
                Err(_) => continue,
                Ok(result) => return Ok(result),
            }
        }
    }

    async fn read(&self) -> TritonFileResult<u64>{
        loop {
            let primary = self.primary_store().await?;
            match primary.read(key).await {
                Err(_) => continue,
                Ok(result) => return Ok(result),
            }
        }
    }

    async fn write(&self,
        req: u64,
        inode: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        #[allow(unused_variables)] flags: i32,
        _lock_owner: Option<u64>) -> TritonFileResult<u32>{
        loop {
            let primary = self.primary_store().await?;
            let backup = self.backup_store().await?;
            let res1 = primary.write(req, inode, fh, offset, data, _write_flags, flags, _lock_owner).await?;
            let res2 = backup.write(req, inode, fh, offset, data, _write_flags, flags, _lock_owner).await?;
        }
        Ok(res1)
    }

    async fn create(&self,  req: &Request,
        parent: u64,
        name: &OsStr,
        mut mode: u32,
        _umask: u32,
        flags: i32) -> TritonFileResult<u64>{
        
    }

    async fn delete(&self) -> TritonFileResult<u64>{
        todo!()
    }

    async fn unlink(&self)-> TritonFileResult<()>{
        todo!()
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