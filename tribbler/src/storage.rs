#![allow(dead_code)]
//! module containing Tribbler storage-related structs and implementations
use async_trait::async_trait;
use bson::Bson;
use fuser::Reply;
use fuser::ReplyData;
use fuser::Session;
use log::error;
use log::info;
use std::cmp::min;
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::os::unix::prelude::FileExt;
use std::os::unix::prelude::OsStrExt;
use std::{collections::HashMap, ffi::OsStr, fs, io::ErrorKind, sync::RwLock};
use tokio::io::BufStream;
use tokio_stream::{Stream, StreamExt};

use crate::error::TritonFileError;
use crate::error::TritonFileResult;
use crate::simple;
use crate::simple::FileKind;
use crate::simple::InodeAttributes;
use crate::simple::SimpleFS;
use crate::simple::check_access;
use crate::simple::clear_suid_sgid;
use crate::simple::time_now;

use fuser::{BackgroundSession, FileAttr, MountOption, Request};

#[derive(Debug, Clone)]

/// A type comprising key-value pair
pub struct KeyValue {
    /// the key
    pub key: String,
    /// the value
    pub value: String,
}

impl KeyValue {
    /// convenience method for constructing a [KeyValue] from two `&str`s
    pub fn new(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
/// A type which represents a pattern that can be used to match on a String.
pub struct Pattern {
    /// exact-match string prefix
    pub prefix: String,
    /// exact-match string suffix
    pub suffix: String,
}

impl Pattern {
    /// this function returns true the provided string matches the prefix and
    /// suffix of the given pattern
    pub fn matches(&self, k: &str) -> bool {
        k.starts_with(&self.prefix) && k.ends_with(&self.suffix)
    }
}

#[derive(Debug, Clone)]
/// A wrapper type around a [Vec<String>]
pub struct List(pub Vec<String>);

#[async_trait]
/// Key-value pair interfaces
/// Default value for all keys is empty string
pub trait KeyString {
    /// Gets a value. If no value set, return [None]
    async fn get(&self, key: &str) -> TritonFileResult<Option<String>>;

    /// Set kv.key to kv.value. return true when no error.
    async fn set(&self, kv: &KeyValue) -> TritonFileResult<bool>;

    /// List all the keys of non-empty pairs where the key matches
    /// the given pattern.
    async fn keys(&self, p: &Pattern) -> TritonFileResult<List>;
}

#[async_trait]
/// Key-list interfaces
pub trait KeyList {
    /// Get the list. Empty if not set.
    async fn list_get(&self, key: &str) -> TritonFileResult<List>;

    /// Append a string to the list. return true when no error.
    async fn list_append(&self, kv: &KeyValue) -> TritonFileResult<bool>;

    /// Removes all elements that are equal to `kv.value` in list `kv.key`
    /// returns the number of elements removed.
    async fn list_remove(&self, kv: &KeyValue) -> TritonFileResult<u32>;

    /// List all the keys of non-empty lists, where the key matches
    /// the given pattern.
    async fn list_keys(&self, p: &Pattern) -> TritonFileResult<List>;
}

#[async_trait]
pub trait ServerFileSystem {
    async fn read(
        &self,
        _req: &Request,
        inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
    ) -> TritonFileResult<String>;

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
    ) -> TritonFileResult<u32>;

    async fn lookup(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<FileAttr>;

    async fn unlink(&mut self, req: &Request, parent: u64, name: &OsStr) -> TritonFileResult<()>;

    async fn create(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
        mut mode: u32,
        _umask: u32,
        flags: i32,
    ) -> TritonFileResult<(FileAttr, u64)>;
}

#[async_trait]
/// A trait representing a storage interface
/// The trait bounds for [KeyString] and [KeyList] respectively represent
/// the functions requires for the single key-value and key-list parts of the
/// storage interface.
pub trait Storage: ServerFileSystem + KeyString + KeyList + Send + Sync {
    /// Returns an auto-incrementing clock. The returned value of each call will
    /// be unique, no smaller than `at_least`, and strictly larger than the
    /// value returned last time, unless it was [u64::MAX]
    async fn clock(&self, at_least: u64) -> TritonFileResult<u64>;
}

/// This is a toy implementation of a backend storage service.
/// The trait definition requires this to be safe to utilize across threads
/// because mutating methods (e.g. [KeyString::set] take `&self` instead of
/// `&mut self`)
#[derive(Debug)]
pub struct RemoteFileSystem {
    kvs: RwLock<HashMap<String, String>>,
    kv_list: RwLock<HashMap<String, List>>,
    clock: RwLock<u64>,
    session: BackgroundSession,
    fs: SimpleFS,
}


impl RemoteFileSystem {
    /// Creates a new instance of [MemStorage]
    pub fn new(num: u32) -> RemoteFileSystem {
        let mut options = vec![MountOption::FSName(format!("fuser{}", num))];
        #[cfg(feature = "abi-7-26")]
        {
            options.push(MountOption::AutoUnmount);
        }
        #[cfg(not(feature = "abi-7-26"))]
        {
            options.push(MountOption::AutoUnmount);
            options.push(MountOption::AllowOther);
        }

        if !fs::metadata(format!("~/Desktop/tmp/{}", num)).is_ok() {
            fs::create_dir(format!("~/Desktop/tmp/{}", num));
        }

        let fs = SimpleFS::new("/tmp/fuser".to_string(), false, true);
        let result = fuser::spawn_mount2(
            fs,
            format!("~/Desktop/tmp/{}", num),
            &options,
        );

        RemoteFileSystem { 
            kvs: RwLock::new(HashMap::new()), 
            kv_list: RwLock::new(HashMap::new()), 
            clock: RwLock::new(0), 
            session: result.unwrap(),
            fs: SimpleFS::new("/tmp/fuser".to_string(), false, true),
        }
    }
}

#[async_trait]
impl KeyString for RemoteFileSystem {
    async fn get(&self, key: &str) -> TritonFileResult<Option<String>> {
        match self.kvs.read().map_err(|e| e.to_string())?.get(key) {
            Some(v) => Ok(Some(v.to_string())),
            None => Ok(None),
        }
    }

    async fn set(&self, kv: &KeyValue) -> TritonFileResult<bool> {
        let mut entry = self.kvs.write().map_err(|e| e.to_string())?;
        if kv.value.is_empty() {
            entry.remove(&kv.key);
        } else {
            entry.insert(kv.key.clone(), kv.value.clone());
        }
        Ok(true)
    }

    async fn keys(&self, p: &Pattern) -> TritonFileResult<List> {
        let result = self
            .kvs
            .read()
            .map_err(|e| e.to_string())?
            .iter()
            .filter(|(k, _)| p.matches(*k))
            .map(|(k, _)| k.to_string())
            .collect::<Vec<String>>();
        Ok(List(result))
    }
}

#[async_trait]
impl KeyList for RemoteFileSystem {
    async fn list_get(&self, key: &str) -> TritonFileResult<List> {
        match self.kv_list.read().map_err(|e| e.to_string())?.get(key) {
            Some(l) => Ok(l.clone()),
            None => Ok(List(vec![])),
        }
    }

    async fn list_append(&self, kv: &KeyValue) -> TritonFileResult<bool> {
        let mut kvl = self.kv_list.write().map_err(|e| e.to_string())?;
        match kvl.get_mut(&kv.key) {
            Some(list) => {
                list.0.push(kv.value.clone());
                Ok(true)
            }
            None => {
                let list = vec![kv.value.clone()];
                kvl.insert(kv.key.clone(), List(list));
                Ok(true)
            }
        }
    }

    async fn list_remove(&self, kv: &KeyValue) -> TritonFileResult<u32> {
        let mut removed = 0;
        let mut kvl = self.kv_list.write().map_err(|e| e.to_string())?;
        kvl.entry(kv.key.clone()).and_modify(|list| {
            let begin_size = list.0.len();
            *list = List(
                list.0
                    .iter()
                    .filter(|val| **val != kv.value)
                    .map(String::from)
                    .collect::<Vec<String>>(),
            );
            let end_size = list.0.len();
            removed = begin_size - end_size;
        });
        if let Some(x) = kvl.get(&kv.key) {
            if x.0.is_empty() {
                kvl.remove(&kv.key);
            }
        };

        Ok(removed as u32)
    }

    async fn list_keys(&self, p: &Pattern) -> TritonFileResult<List> {
        let mut result = vec![];
        self.kv_list
            .read()
            .map_err(|e| e.to_string())?
            .iter()
            .filter(|(k, _)| p.matches(*k))
            .for_each(|(v, _)| result.push((*v).clone()));
        result.sort();
        Ok(List(result))
    }
}

#[async_trait]
impl ServerFileSystem for RemoteFileSystem {
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
        let fs = &self.fs;
        info!(
            "read() called on {:?} offset={:?} size={:?}",
            inode, offset, size
        );
        assert!(offset >= 0);
        if !fs.check_file_handle_read(fh) {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::EACCES)));
        }

        let path = fs.content_path(inode);
        if let Ok(file) = File::open(&path) {
            let file_size = file.metadata().unwrap().len();
            // Could underflow if file length is less than local_start
            let read_size = min(size, file_size.saturating_sub(offset as u64) as u32);

            let mut buffer = vec![0; read_size as usize];
            file.read_exact_at(&mut buffer, offset as u64).unwrap();
            return Ok(serde_json::to_string(&buffer).unwrap());
        } else {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::ENOENT)));
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
        let fs = &self.fs;

        info!("write() called with {:?} size={:?}", inode, data.len());
        assert!(offset >= 0);
        if !fs.check_file_handle_write(fh) {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::EACCES)));
        }

        let path = fs.content_path(inode);
        if let Ok(mut file) = OpenOptions::new().write(true).open(&path) {
            file.seek(SeekFrom::Start(offset as u64)).unwrap();
            file.write_all(data).unwrap();

            let mut attrs = fs.get_inode(inode).unwrap();
            attrs.last_metadata_changed = time_now();
            attrs.last_modified = time_now();
            if data.len() + offset as usize > attrs.size as usize {
                attrs.size = (data.len() + offset as usize) as u64;
            }
            // #[cfg(feature = "abi-7-31")]
            // if flags & FUSE_WRITE_KILL_PRIV as i32 != 0 {
            //     clear_suid_sgid(&mut attrs);
            // }
            // XXX: In theory we should only need to do this when WRITE_KILL_PRIV is set for 7.31+
            // However, xfstests fail in that case
            clear_suid_sgid(&mut attrs);
            fs.write_inode(&attrs);

            return Ok(data.len() as u32);
        } else {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::EBADF)));
        }
    }

    async fn lookup(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<FileAttr>{
        let fs = &self.fs;
        if name.len() > simple::MAX_NAME_LENGTH as usize {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::ENAMETOOLONG)));
        }
        let parent_attrs = fs.get_inode(parent).unwrap();
        if !check_access(
            parent_attrs.uid,
            parent_attrs.gid,
            parent_attrs.mode,
            req.uid(),
            req.gid(),
            libc::X_OK,
        ) {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::EACCES)));
        }

        match fs.lookup_name(parent, name) {
            Ok(attrs) => Ok(attrs.into()),
            Err(error_code) => Err(Box::new(TritonFileError::UserInterfaceError(error_code))),
        }
    }

    async fn unlink(&mut self, req: &Request, parent: u64, name: &OsStr) -> TritonFileResult<()>{
        let fs = &self.fs;

        info!("unlink() called with {:?} {:?}", parent, name);
        let mut attrs = match fs.lookup_name(parent, name) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Err(Box::new(TritonFileError::UserInterfaceError(error_code)));
            }
        };

        let mut parent_attrs = match fs.get_inode(parent) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Err(Box::new(TritonFileError::UserInterfaceError(error_code)));
            }
        };

        if !check_access(
            parent_attrs.uid,
            parent_attrs.gid,
            parent_attrs.mode,
            req.uid(),
            req.gid(),
            libc::W_OK,
        ) {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::EACCES)));
        }

        let uid = req.uid();
        // "Sticky bit" handling
        if parent_attrs.mode & libc::S_ISVTX as u16 != 0
            && uid != 0
            && uid != parent_attrs.uid
            && uid != attrs.uid
        {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::EACCES)));
        }

        parent_attrs.last_metadata_changed = time_now();
        parent_attrs.last_modified = time_now();
        fs.write_inode(&parent_attrs);

        attrs.hardlinks -= 1;
        attrs.last_metadata_changed = time_now();
        fs.write_inode(&attrs);
        fs.gc_inode(&attrs);

        let mut entries = fs.get_directory_content(parent).unwrap();
        entries.remove(name.as_bytes());
        fs.write_directory_content(parent, entries);

        Ok(())
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
        let fs = &self.fs;
        info!("create() called with {:?} {:?}", parent, name);
        if fs.lookup_name(parent, name).is_ok() {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::EEXIST)));
        }

        let (read, write) = match flags & libc::O_ACCMODE {
            libc::O_RDONLY => (true, false),
            libc::O_WRONLY => (false, true),
            libc::O_RDWR => (true, true),
            // Exactly one access mode flag must be specified
            _ => {
                return Err(Box::new(TritonFileError::UserInterfaceError(libc::EINVAL)));
            }
        };

        let mut parent_attrs = match fs.get_inode(parent) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Err(Box::new(TritonFileError::UserInterfaceError(error_code)));
            }
        };

        if !check_access(
            parent_attrs.uid,
            parent_attrs.gid,
            parent_attrs.mode,
            req.uid(),
            req.gid(),
            libc::W_OK,
        ) {
            return Err(Box::new(TritonFileError::UserInterfaceError(libc::EACCES)));
        }
        parent_attrs.last_modified = time_now();
        parent_attrs.last_metadata_changed = time_now();
        fs.write_inode(&parent_attrs);

        if req.uid() != 0 {
            mode &= !(libc::S_ISUID | libc::S_ISGID) as u32;
        }

        let inode = fs.allocate_next_inode();
        let attrs = InodeAttributes {
            inode,
            open_file_handles: 1,
            size: 0,
            last_accessed: time_now(),
            last_modified: time_now(),
            last_metadata_changed: time_now(),
            kind: simple::as_file_kind(mode),
            mode: fs.creation_mode(mode),
            hardlinks: 1,
            uid: req.uid(),
            gid: simple::creation_gid(&parent_attrs, req.gid()),
            xattrs: Default::default(),
        };
        fs.write_inode(&attrs);
        File::create(fs.content_path(inode)).unwrap();

        if simple::as_file_kind(mode) == FileKind::Directory {
            let mut entries = BTreeMap::new();
            entries.insert(b".".to_vec(), (inode, FileKind::Directory));
            entries.insert(b"..".to_vec(), (parent, FileKind::Directory));
            fs.write_directory_content(inode, entries);
        }

        let mut entries = fs.get_directory_content(parent).unwrap();
        entries.insert(name.as_bytes().to_vec(), (inode, attrs.kind));
        fs.write_directory_content(parent, entries);

        // TODO: implement flags

        Ok((attrs.into(), fs.allocate_next_file_handle(read, write)))
    }
}

#[async_trait]
impl Storage for RemoteFileSystem {
    async fn clock(&self, at_least: u64) -> TritonFileResult<u64> {
        let mut clk = self.clock.write().map_err(|e| e.to_string())?;
        if *clk < at_least {
            *clk = at_least
        }

        let ret = *clk;

        if *clk < u64::MAX {
            *clk += 1;
        }
        Ok(ret)
    }
}

#[async_trait]
/// Bin Storage interface
pub trait BinStorage: Send + Sync {
    /// Fetch a [Storage] bin based on the given bin name.
    async fn bin(&self, name: &str) -> TritonFileResult<Box<dyn Storage>>;
}

#[cfg(test)]
mod test {
    use crate::{
        error::TritonFileResult,
        storage::{KeyValue, Pattern, Storage},
    };

    use super::{KeyList, KeyString, RemoteFileSystem};

    async fn setup_test_storage() -> RemoteFileSystem {
        let storage = RemoteFileSystem::new(1);
        storage
            .set(&KeyValue {
                key: "test".to_string(),
                value: "test-value".to_string(),
            })
            .await
            .unwrap();
        storage
            .list_append(&KeyValue {
                key: "test".to_string(),
                value: "test-value".to_string(),
            })
            .await
            .unwrap();
        storage
    }

    #[tokio::test]
    async fn storage_get_set() -> TritonFileResult<()> {
        let storage = RemoteFileSystem::new(1);
        assert_eq!(
            true,
            storage
                .set(&KeyValue {
                    key: "test".to_string(),
                    value: "test-value".to_string()
                })
                .await?
        );
        assert_eq!(Some("test-value".to_string()), storage.get("test").await?);
        Ok(())
    }

//     #[tokio::test]
//     async fn storage_get_empty() -> TritonFileResult<()> {
//         let storage = setup_test_storage().await;
//         assert_eq!(None, storage.get("test2").await?);
//         Ok(())
//     }

//     #[tokio::test]
//     async fn storage_keys() {
//         let storage = setup_test_storage().await;
//         let p1 = Pattern {
//             prefix: "test".to_string(),
//             suffix: "test".to_string(),
//         };
//         let p2 = Pattern {
//             prefix: "".to_string(),
//             suffix: "test".to_string(),
//         };
//         let p3 = Pattern {
//             prefix: "test".to_string(),
//             suffix: "".to_string(),
//         };
//         let p4 = Pattern {
//             prefix: "wrong".to_string(),
//             suffix: "right".to_string(),
//         };
//         let p5 = Pattern {
//             prefix: "".to_string(),
//             suffix: "".to_string(),
//         };
//         assert_eq!(1, storage.keys(&p1).await.unwrap().0.len());
//         assert_eq!(1, storage.keys(&p2).await.unwrap().0.len());
//         assert_eq!(1, storage.keys(&p3).await.unwrap().0.len());
//         assert_eq!(0, storage.keys(&p4).await.unwrap().0.len());
//         assert_eq!(1, storage.keys(&p5).await.unwrap().0.len());
//     }

//     #[tokio::test]
//     async fn storage_keys_unset() {
//         let s = setup_test_storage().await;
//         assert_eq!(1, s.keys(&Pattern::default()).await.unwrap().0.len());
//         let _ = s.set(&KeyValue::new("test", "")).await.unwrap();
//         assert_eq!(0, s.keys(&Pattern::default()).await.unwrap().0.len())
//     }

//     #[tokio::test]
//     async fn storage_list_keys_unset() {
//         let s = setup_test_storage().await;
//         assert_eq!(1, s.list_keys(&Pattern::default()).await.unwrap().0.len());
//         let _ = s
//             .list_remove(&KeyValue::new("test", "test-value"))
//             .await
//             .unwrap();
//         assert_eq!(0, s.list_keys(&Pattern::default()).await.unwrap().0.len())
//     }

//     #[tokio::test]
//     async fn storage_get_list() {
//         let storage = setup_test_storage().await;
//         assert_eq!("test-value", storage.list_get("test").await.unwrap().0[0]);
//     }

//     #[tokio::test]
//     async fn storage_get_list_empty() {
//         let storage = setup_test_storage().await;
//         assert_eq!(0, storage.list_get("test2").await.unwrap().0.len());
//     }

//     #[tokio::test]
//     async fn storage_get_list_append() -> TritonFileResult<()> {
//         let storage = setup_test_storage().await;
//         let res = storage
//             .list_append(&KeyValue {
//                 key: "test".to_string(),
//                 value: "val2".to_string(),
//             })
//             .await?;
//         assert_eq!(true, res);
//         assert_eq!(2, storage.list_get("test").await.unwrap().0.len());
//         Ok(())
//     }

//     #[tokio::test]
//     async fn storage_get_list_remove() {
//         let storage = setup_test_storage().await;
//         let kv = KeyValue {
//             key: "test".to_string(),
//             value: "val2".to_string(),
//         };
//         assert_eq!(true, storage.list_append(&kv).await.unwrap());
//         assert_eq!(true, storage.list_append(&kv).await.unwrap());
//         assert_eq!(true, storage.list_append(&kv).await.unwrap());
//         assert_eq!(3, storage.list_remove(&kv).await.unwrap());
//         println!("{:?}", storage.list_get("test").await.unwrap().0);
//         assert_eq!("test-value", storage.list_get("test").await.unwrap().0[0]);
//     }

//     #[tokio::test]
//     async fn storage_list_keys() {
//         let storage = setup_test_storage().await;
//         let p1 = Pattern {
//             prefix: "test".to_string(),
//             suffix: "test".to_string(),
//         };
//         let p2 = Pattern {
//             prefix: "".to_string(),
//             suffix: "test".to_string(),
//         };
//         let p3 = Pattern {
//             prefix: "test".to_string(),
//             suffix: "".to_string(),
//         };
//         let p4 = Pattern {
//             prefix: "wrong".to_string(),
//             suffix: "right".to_string(),
//         };
//         let p5 = Pattern {
//             prefix: "".to_string(),
//             suffix: "".to_string(),
//         };
//         assert_eq!(1, storage.list_keys(&p1).await.unwrap().0.len());
//         assert_eq!(1, storage.list_keys(&p2).await.unwrap().0.len());
//         assert_eq!(1, storage.list_keys(&p3).await.unwrap().0.len());
//         assert_eq!(0, storage.list_keys(&p4).await.unwrap().0.len());
//         assert_eq!(1, storage.list_keys(&p5).await.unwrap().0.len());
//     }

//     #[tokio::test]
//     async fn clock_at_least() {
//         let storage = setup_test_storage().await;
//         assert_eq!(1234, storage.clock(1234).await.unwrap());
//     }

//     #[tokio::test]
//     async fn clock_ge() {
//         let storage = setup_test_storage().await;
//         let c1 = storage.clock(1234).await.unwrap();
//         let c2 = storage.clock(0).await.unwrap();
//         assert_eq!(true, c2 > c1);
//     }
}
