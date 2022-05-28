#![allow(dead_code)]
//! module containing Tribbler storage-related structs and implementations
use async_trait::async_trait;
use bson::Bson;
use fuser::consts::FOPEN_DIRECT_IO;
use fuser::Reply;
use fuser::ReplyData;
use fuser::Session;
use fuser::TimeOrNow;
use fuser::TimeOrNow::Now;
use libc::c_int;
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
use std::time::SystemTime;
use std::{collections::HashMap, ffi::OsStr, fs, io::ErrorKind, sync::RwLock};
use tokio::io::BufStream;
use tokio_stream::{Stream, StreamExt};

use crate::error;
use crate::error::TritonFileError;
use crate::error::TritonFileResult;
use crate::error::SUCCESS;
use crate::simple;
use crate::simple::check_access;
use crate::simple::clear_suid_sgid;
use crate::simple::get_groups;
use crate::simple::time_from_system_time;
use crate::simple::time_now;
use crate::simple::xattr_access_check;
use crate::simple::FileKind;
use crate::simple::InodeAttributes;
use crate::simple::SimpleFS;
use crate::simple::FMODE_EXEC;

use fuser::{BackgroundSession, FileAttr, MountOption, Request};

#[derive(Debug, Clone)]

pub struct FileRequest {
    pub uid: u32,
    pub gid: u32,
    pub pid: u32,
}

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
        _req: &FileRequest,
        inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
    ) -> TritonFileResult<(Option<String>, c_int)>;

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
    ) -> TritonFileResult<(Option<u32>, c_int)>;

    async fn lookup(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)>;

    async fn unlink(&self, req: &FileRequest, parent: u64, name: &OsStr)
        -> TritonFileResult<c_int>;

    async fn create(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
        mut mode: u32,
        _umask: u32,
        flags: i32,
    ) -> TritonFileResult<(Option<(FileAttr, u64)>, c_int)>;

    async fn getattr(
        &self,
        _req: &FileRequest,
        ino: u64,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)>;

    async fn open(
        &self,
        _req: &FileRequest,
        _ino: u64,
        _flags: i32,
    ) -> TritonFileResult<(Option<(u64, u32)>, c_int)>;

    async fn release(
        &self,
        _req: &FileRequest,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
    ) -> TritonFileResult<(c_int)>;

    async fn setxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        name: &OsStr,
        _value: &[u8],
        flags: i32,
        position: u32,
    ) -> TritonFileResult<(c_int)>;

    //reply Vec<u8> as string
    async fn getxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        name: &OsStr,
        size: u32,
    ) -> TritonFileResult<(Option<(String, u32)>, c_int)>;

    async fn listxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        size: u32,
    ) -> TritonFileResult<(Option<(String, u32)>, c_int)>;

    async fn access(&self, _req: &FileRequest, ino: u64, mask: i32) -> TritonFileResult<(c_int)>;

    async fn rename(
        &self,
        _req: &FileRequest,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        flags: u32,
    ) -> TritonFileResult<(c_int)>;

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
    ) -> TritonFileResult<(Option<FileAttr>, c_int)>;
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

        if !fs::metadata(format!("tmp/{}", num)).is_ok() {
            fs::create_dir_all(format!("tmp/{}", num)).unwrap();
        }

        RemoteFileSystem {
            kvs: RwLock::new(HashMap::new()),
            kv_list: RwLock::new(HashMap::new()),
            clock: RwLock::new(0),
            fs: SimpleFS::new(format!("tmp/{}", num), false, false),
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
        _req: &FileRequest,
        inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
    ) -> TritonFileResult<(Option<String>, c_int)> {
        let fs = &self.fs;
        info!(
            "read() called on {:?} offset={:?} size={:?}",
            inode, offset, size
        );
        assert!(offset >= 0);
        if !fs.check_file_handle_read(fh) {
            return Ok((None, libc::EACCES));
        }

        let path = fs.content_path(inode);
        if let Ok(file) = File::open(&path) {
            let file_size = file.metadata().unwrap().len();
            // Could underflow if file length is less than local_start
            let read_size = min(size, file_size.saturating_sub(offset as u64) as u32);

            let mut buffer = vec![0; read_size as usize];
            file.read_exact_at(&mut buffer, offset as u64).unwrap();
            return Ok((
                Some(serde_json::to_string(&buffer).unwrap()),
                error::SUCCESS,
            ));
        } else {
            return Ok((None, libc::ENOENT));
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
        let fs = &self.fs;

        info!("write() called with {:?} size={:?}", inode, data.len());
        assert!(offset >= 0);
        if !fs.check_file_handle_write(fh) {
            return Ok((None, libc::EACCES));
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

            return Ok((Some(data.len() as u32), error::SUCCESS));
        } else {
            return Ok((None, libc::EBADF));
        }
    }

    async fn lookup(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)> {
        let fs = &self.fs;
        if name.len() > simple::MAX_NAME_LENGTH as usize {
            return Err(Box::new(TritonFileError::UserInterfaceError(
                libc::ENAMETOOLONG,
            )));
        }
        let parent_attrs = fs.get_inode(parent).unwrap();
        if !check_access(
            parent_attrs.uid,
            parent_attrs.gid,
            parent_attrs.mode,
            req.uid,
            req.gid,
            libc::X_OK,
        ) {
            return Ok((None, libc::EACCES));
        }

        match fs.lookup_name(parent, name) {
            Ok(attrs) => Ok((Some(attrs.into()), error::SUCCESS)),
            Err(error_code) => Ok((None, error_code)),
        }
    }

    async fn unlink(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<c_int> {
        let fs = &self.fs;

        info!("unlink() called with {:?} {:?}", parent, name);
        let mut attrs = match fs.lookup_name(parent, name) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Ok(error_code);
            }
        };

        let mut parent_attrs = match fs.get_inode(parent) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Ok(error_code);
            }
        };

        if !check_access(
            parent_attrs.uid,
            parent_attrs.gid,
            parent_attrs.mode,
            req.uid,
            req.gid,
            libc::W_OK,
        ) {
            return Ok(libc::EACCES);
        }

        let uid = req.uid;
        // "Sticky bit" handling
        if parent_attrs.mode & libc::S_ISVTX as u16 != 0
            && uid != 0
            && uid != parent_attrs.uid
            && uid != attrs.uid
        {
            return Ok(libc::EACCES);
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

        Ok(error::SUCCESS)
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
        let fs = &self.fs;
        info!("create() called with {:?} {:?}", parent, name);
        if fs.lookup_name(parent, name).is_ok() {
            return Ok((None, libc::EEXIST));
        }

        let (read, write) = match flags & libc::O_ACCMODE {
            libc::O_RDONLY => (true, false),
            libc::O_WRONLY => (false, true),
            libc::O_RDWR => (true, true),
            // Exactly one access mode flag must be specified
            _ => {
                return Ok((None, libc::EINVAL));
            }
        };

        let mut parent_attrs = match fs.get_inode(parent) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Ok((None, error_code));
            }
        };

        if !check_access(
            parent_attrs.uid,
            parent_attrs.gid,
            parent_attrs.mode,
            req.uid,
            req.gid,
            libc::W_OK,
        ) {
            return Ok((None, libc::EACCES));
        }
        parent_attrs.last_modified = time_now();
        parent_attrs.last_metadata_changed = time_now();
        fs.write_inode(&parent_attrs);

        if req.uid != 0 {
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
            uid: req.uid,
            gid: simple::creation_gid(&parent_attrs, req.gid),
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

        Ok((
            Some((attrs.into(), fs.allocate_next_file_handle(read, write))),
            error::SUCCESS,
        ))
    }

    async fn getattr(
        &self,
        _req: &FileRequest,
        ino: u64,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)> {
        let fs = &self.fs;
        match fs.get_inode(ino) {
            Ok(attrs) => Ok((Some(attrs.into()), SUCCESS)),
            Err(error_code) => Ok((None, error_code)),
        }
    }

    async fn open(
        &self,
        req: &FileRequest,
        inode: u64,
        flags: i32,
    ) -> TritonFileResult<(Option<(u64, u32)>, c_int)> {
        let fs = &self.fs;

        info!("open() called for {:?}", inode);
        let (access_mask, read, write) = match flags & libc::O_ACCMODE {
            libc::O_RDONLY => {
                // Behavior is undefined, but most filesystems return EACCES
                if flags & libc::O_TRUNC != 0 {
                    return Ok((None, libc::EACCES));
                }
                if flags & FMODE_EXEC != 0 {
                    // Open is from internal exec syscall
                    (libc::X_OK, true, false)
                } else {
                    (libc::R_OK, true, false)
                }
            }
            libc::O_WRONLY => (libc::W_OK, false, true),
            libc::O_RDWR => (libc::R_OK | libc::W_OK, true, true),
            // Exactly one access mode flag must be specified
            _ => {
                return Ok((None, libc::EINVAL));
            }
        };

        match fs.get_inode(inode) {
            Ok(mut attr) => {
                if check_access(attr.uid, attr.gid, attr.mode, req.uid, req.gid, access_mask) {
                    attr.open_file_handles += 1;
                    fs.write_inode(&attr);
                    let open_flags = if fs.direct_io { FOPEN_DIRECT_IO } else { 0 };
                    return Ok((
                        Some((fs.allocate_next_file_handle(read, write), open_flags)),
                        SUCCESS,
                    ));
                } else {
                    return Ok((None, libc::EACCES));
                }
            }
            Err(error_code) => return Ok((None, error_code)),
        }
    }

    async fn release(
        &self,
        _req: &FileRequest,
        inode: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
    ) -> TritonFileResult<(c_int)> {
        let fs = &self.fs;
        if let Ok(mut attrs) = fs.get_inode(inode) {
            attrs.open_file_handles -= 1;
        }
        return Ok(SUCCESS);
    }

    async fn setxattr(
        &self,
        request: &FileRequest,
        inode: u64,
        key: &OsStr,
        value: &[u8],
        flags: i32,
        position: u32,
    ) -> TritonFileResult<(c_int)> {
        let fs = &self.fs;
        if let Ok(mut attrs) = fs.get_inode(inode) {
            if let Err(error) = xattr_access_check(key.as_bytes(), libc::W_OK, &attrs, request) {
                return Ok(error);
            }

            attrs.xattrs.insert(key.as_bytes().to_vec(), value.to_vec());
            attrs.last_metadata_changed = time_now();
            fs.write_inode(&attrs);
            return Ok(SUCCESS);
        } else {
            return Ok(libc::EBADF);
        }
    }

    //reply Vec<u8> as string
    async fn getxattr(
        &self,
        request: &FileRequest,
        inode: u64,
        key: &OsStr,
        size: u32,
    ) -> TritonFileResult<(Option<(String, u32)>, c_int)> {
        let fs = &self.fs;

        if let Ok(attrs) = fs.get_inode(inode) {
            if let Err(error) = xattr_access_check(key.as_bytes(), libc::R_OK, &attrs, request) {
                return Ok((None, error));
            }

            /// check size to get usize
            if let Some(data) = attrs.xattrs.get(key.as_bytes()) {
                if size == 0 {
                    return Ok((
                        Some((serde_json::to_string("").unwrap(), data.len() as u32)),
                        SUCCESS,
                    ));
                } else if data.len() <= size as usize {
                    return Ok((
                        Some((serde_json::to_string(data).unwrap(), data.len() as u32)),
                        SUCCESS,
                    ));
                } else {
                    return Ok((None, libc::ERANGE));
                }
            } else {
                #[cfg(target_os = "linux")]
                return Ok((None, libc::ENODATA));
                #[cfg(not(target_os = "linux"))]
                return Ok((None, libc::ENODATA));
            }
        } else {
            return Ok((None, libc::EBADF));
        }
    }

    async fn listxattr(
        &self,
        _req: &FileRequest,
        inode: u64,
        size: u32,
    ) -> TritonFileResult<(Option<(String, u32)>, c_int)> {
        let fs = &self.fs;
        if let Ok(attrs) = fs.get_inode(inode) {
            let mut bytes = vec![];
            // Convert to concatenated null-terminated strings
            for key in attrs.xattrs.keys() {
                bytes.extend(key);
                bytes.push(0);
            }
            if size == 0 {
                return Ok((
                    Some((serde_json::to_string("").unwrap(), bytes.len() as u32)),
                    SUCCESS,
                ));
            } else if bytes.len() <= size as usize {
                return Ok((
                    Some((serde_json::to_string(&bytes).unwrap(), bytes.len() as u32)),
                    SUCCESS,
                ));
            } else {
                return Ok((None, libc::ERANGE));
            }
        } else {
            return Ok((None, libc::EBADF));
        }
    }

    async fn access(&self, req: &FileRequest, inode: u64, mask: i32) -> TritonFileResult<(c_int)> {
        let fs = &self.fs;

        info!("access() called with {:?} {:?}", inode, mask);
        match fs.get_inode(inode) {
            Ok(attr) => {
                if check_access(attr.uid, attr.gid, attr.mode, req.uid, req.gid, mask) {
                    return Ok(SUCCESS);
                } else {
                    return Ok(libc::EACCES);
                }
            }
            Err(error_code) => return Ok(error_code),
        }
    }

    async fn rename(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
        new_parent: u64,
        new_name: &OsStr,
        flags: u32,
    ) -> TritonFileResult<(c_int)> {
        let fs = &self.fs;

        let mut inode_attrs = match fs.lookup_name(parent, name) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Ok(error_code);
            }
        };

        let mut parent_attrs = match fs.get_inode(parent) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Ok(error_code);
            }
        };

        if !check_access(
            parent_attrs.uid,
            parent_attrs.gid,
            parent_attrs.mode,
            req.uid,
            req.gid,
            libc::W_OK,
        ) {
            return Ok(libc::EACCES);
        }

        // "Sticky bit" handling
        if parent_attrs.mode & libc::S_ISVTX as u16 != 0
            && req.uid != 0
            && req.uid != parent_attrs.uid
            && req.uid != inode_attrs.uid
        {
            return Ok(libc::EACCES);
        }

        let mut new_parent_attrs = match fs.get_inode(new_parent) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Ok(error_code);
            }
        };

        if !check_access(
            new_parent_attrs.uid,
            new_parent_attrs.gid,
            new_parent_attrs.mode,
            req.uid,
            req.gid,
            libc::W_OK,
        ) {
            return Ok(libc::EACCES);
        }

        // "Sticky bit" handling in new_parent
        if new_parent_attrs.mode & libc::S_ISVTX as u16 != 0 {
            if let Ok(existing_attrs) = fs.lookup_name(new_parent, new_name) {
                if req.uid != 0 && req.uid != new_parent_attrs.uid && req.uid != existing_attrs.uid
                {
                    return Ok(libc::EACCES);
                }
            }
        }

        #[cfg(target_os = "linux")]
        if flags & libc::RENAME_EXCHANGE as u32 != 0 {
            let mut new_inode_attrs = match self.lookup_name(new_parent, new_name) {
                Ok(attrs) => attrs,
                Err(error_code) => {
                    return Ok(error_code);
                }
            };

            let mut entries = self.get_directory_content(new_parent).unwrap();
            entries.insert(
                new_name.as_bytes().to_vec(),
                (inode_attrs.inode, inode_attrs.kind),
            );
            self.write_directory_content(new_parent, entries);

            let mut entries = self.get_directory_content(parent).unwrap();
            entries.insert(
                name.as_bytes().to_vec(),
                (new_inode_attrs.inode, new_inode_attrs.kind),
            );
            self.write_directory_content(parent, entries);

            parent_attrs.last_metadata_changed = time_now();
            parent_attrs.last_modified = time_now();
            self.write_inode(&parent_attrs);
            new_parent_attrs.last_metadata_changed = time_now();
            new_parent_attrs.last_modified = time_now();
            self.write_inode(&new_parent_attrs);
            inode_attrs.last_metadata_changed = time_now();
            self.write_inode(&inode_attrs);
            new_inode_attrs.last_metadata_changed = time_now();
            self.write_inode(&new_inode_attrs);

            if inode_attrs.kind == FileKind::Directory {
                let mut entries = self.get_directory_content(inode_attrs.inode).unwrap();
                entries.insert(b"..".to_vec(), (new_parent, FileKind::Directory));
                self.write_directory_content(inode_attrs.inode, entries);
            }
            if new_inode_attrs.kind == FileKind::Directory {
                let mut entries = self.get_directory_content(new_inode_attrs.inode).unwrap();
                entries.insert(b"..".to_vec(), (parent, FileKind::Directory));
                self.write_directory_content(new_inode_attrs.inode, entries);
            }

            return Ok(SUCCESS);
        }

        // Only overwrite an existing directory if it's empty
        if let Ok(new_name_attrs) = fs.lookup_name(new_parent, new_name) {
            if new_name_attrs.kind == FileKind::Directory
                && fs
                    .get_directory_content(new_name_attrs.inode)
                    .unwrap()
                    .len()
                    > 2
            {
                return Ok(libc::ENOTEMPTY);
            }
        }

        // Only move an existing directory to a new parent, if we have write access to it,
        // because that will change the ".." link in it
        if inode_attrs.kind == FileKind::Directory
            && parent != new_parent
            && !check_access(
                inode_attrs.uid,
                inode_attrs.gid,
                inode_attrs.mode,
                req.uid,
                req.gid,
                libc::W_OK,
            )
        {
            return Ok(libc::EACCES);
        }

        // If target already exists decrement its hardlink count
        if let Ok(mut existing_inode_attrs) = fs.lookup_name(new_parent, new_name) {
            let mut entries = fs.get_directory_content(new_parent).unwrap();
            entries.remove(new_name.as_bytes());
            fs.write_directory_content(new_parent, entries);

            if existing_inode_attrs.kind == FileKind::Directory {
                existing_inode_attrs.hardlinks = 0;
            } else {
                existing_inode_attrs.hardlinks -= 1;
            }
            existing_inode_attrs.last_metadata_changed = time_now();
            fs.write_inode(&existing_inode_attrs);
            fs.gc_inode(&existing_inode_attrs);
        }

        let mut entries = fs.get_directory_content(parent).unwrap();
        entries.remove(name.as_bytes());
        fs.write_directory_content(parent, entries);

        let mut entries = fs.get_directory_content(new_parent).unwrap();
        entries.insert(
            new_name.as_bytes().to_vec(),
            (inode_attrs.inode, inode_attrs.kind),
        );
        fs.write_directory_content(new_parent, entries);

        parent_attrs.last_metadata_changed = time_now();
        parent_attrs.last_modified = time_now();
        fs.write_inode(&parent_attrs);
        new_parent_attrs.last_metadata_changed = time_now();
        new_parent_attrs.last_modified = time_now();
        fs.write_inode(&new_parent_attrs);
        inode_attrs.last_metadata_changed = time_now();
        fs.write_inode(&inode_attrs);

        if inode_attrs.kind == FileKind::Directory {
            let mut entries = fs.get_directory_content(inode_attrs.inode).unwrap();
            entries.insert(b"..".to_vec(), (new_parent, FileKind::Directory));
            fs.write_directory_content(inode_attrs.inode, entries);
        }

        return Ok(SUCCESS);
    }

    async fn setattr(
        &self,
        req: &FileRequest,
        inode: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<TimeOrNow>,
        mtime: Option<TimeOrNow>,
        _ctime: Option<SystemTime>,
        fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        flags: Option<u32>,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)> {
        let fs = &self.fs;

        let mut attrs = match fs.get_inode(inode) {
            Ok(attrs) => attrs,
            Err(error_code) => {
                return Ok((None, error_code));
            }
        };

        if let Some(mode) = mode {
            info!("chmod() called with {:?}, {:o}", inode, mode);
            if req.uid != 0 && req.uid != attrs.uid {
                return Ok((None, libc::EPERM));
            }
            if req.uid != 0 && req.gid != attrs.gid && !get_groups(req.pid).contains(&attrs.gid) {
                // If SGID is set and the file belongs to a group that the caller is not part of
                // then the SGID bit is suppose to be cleared during chmod
                attrs.mode = (mode & !libc::S_ISGID as u32) as u16;
            } else {
                attrs.mode = mode as u16;
            }
            attrs.last_metadata_changed = time_now();
            fs.write_inode(&attrs);
            return Ok((Some(attrs.into()), SUCCESS));
        }

        if uid.is_some() || gid.is_some() {
            info!("chown() called with {:?} {:?} {:?}", inode, uid, gid);
            if let Some(gid) = gid {
                // Non-root users can only change gid to a group they're in
                if req.uid != 0 && !get_groups(req.pid).contains(&gid) {
                    return Ok((None, libc::EPERM));
                }
            }
            if let Some(uid) = uid {
                if req.uid != 0
                    // but no-op changes by the owner are not an error
                    && !(uid == attrs.uid && req.uid == attrs.uid)
                {
                    return Ok((None, libc::EPERM));
                }
            }
            // Only owner may change the group
            if gid.is_some() && req.uid != 0 && req.uid != attrs.uid {
                return Ok((None, libc::EPERM));
            }

            if attrs.mode & (libc::S_IXUSR | libc::S_IXGRP | libc::S_IXOTH) as u16 != 0 {
                // SUID & SGID are suppose to be cleared when chown'ing an executable file
                clear_suid_sgid(&mut attrs);
            }

            if let Some(uid) = uid {
                attrs.uid = uid;
                // Clear SETUID on owner change
                attrs.mode &= !libc::S_ISUID as u16;
            }
            if let Some(gid) = gid {
                attrs.gid = gid;
                // Clear SETGID unless user is root
                if req.uid != 0 {
                    attrs.mode &= !libc::S_ISGID as u16;
                }
            }
            attrs.last_metadata_changed = time_now();
            fs.write_inode(&attrs);
            return Ok((Some(attrs.into()), SUCCESS));
        }

        if let Some(size) = size {
            info!("truncate() called with {:?} {:?}", inode, size);
            if let Some(handle) = fh {
                // If the file handle is available, check access locally.
                // This is important as it preserves the semantic that a file handle opened
                // with W_OK will never fail to truncate, even if the file has been subsequently
                // chmod'ed
                if fs.check_file_handle_write(handle) {
                    if let Err(error_code) = fs.truncate(inode, size, 0, 0) {
                        return Ok((None, error_code));
                    }
                } else {
                    return Ok((None, libc::EACCES));
                }
            } else if let Err(error_code) = fs.truncate(inode, size, req.uid, req.gid) {
                return Ok((None, error_code));
            }
        }

        let now = time_now();
        if let Some(atime) = atime {
            info!("utimens() called with {:?}, atime={:?}", inode, atime);

            if attrs.uid != req.uid && req.uid != 0 && atime != Now {
                return Ok((None, libc::EPERM));
            }

            if attrs.uid != req.uid
                && !check_access(
                    attrs.uid,
                    attrs.gid,
                    attrs.mode,
                    req.uid,
                    req.gid,
                    libc::W_OK,
                )
            {
                return Ok((None, libc::EACCES));
            }

            attrs.last_accessed = match atime {
                TimeOrNow::SpecificTime(time) => time_from_system_time(&time),
                Now => now,
            };
            attrs.last_metadata_changed = now;
            fs.write_inode(&attrs);
        }
        if let Some(mtime) = mtime {
            info!("utimens() called with {:?}, mtime={:?}", inode, mtime);

            if attrs.uid != req.uid && req.uid != 0 && mtime != Now {
                return Ok((None, libc::EPERM));
            }

            if attrs.uid != req.uid
                && !check_access(
                    attrs.uid,
                    attrs.gid,
                    attrs.mode,
                    req.uid,
                    req.gid,
                    libc::W_OK,
                )
            {
                return Ok((None, libc::EACCES));
            }

            attrs.last_modified = match mtime {
                TimeOrNow::SpecificTime(time) => time_from_system_time(&time),
                Now => now,
            };
            attrs.last_metadata_changed = now;
            fs.write_inode(&attrs);
        }

        let attrs = fs.get_inode(inode).unwrap();
        return Ok((Some(attrs.into()), SUCCESS));
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
    use std::fs;

    use crate::error::TritonFileResult;

    use super::{KeyList, KeyString, KeyValue, RemoteFileSystem};

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
    async fn test_create_dir() -> TritonFileResult<()> {
        if !fs::metadata(format!("tmp/{}", 2)).is_ok() {
            println!("The directory is not found");
            fs::create_dir_all(format!("tmp/{}", 2))?;
        }
        Ok(())
    }

    // #[tokio::test]
    // async fn storage_get_empty() -> TritonFileResult<()> {
    //     let storage = setup_test_storage().await;
    //     assert_eq!(None, storage.get("test2").await?);
    //     Ok(())
    // }

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
