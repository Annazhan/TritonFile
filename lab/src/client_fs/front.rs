#![allow(clippy::needless_return)]

use clap::{crate_version, Arg, Command};
use fuser::consts::FOPEN_DIRECT_IO;
#[cfg(feature = "abi-7-26")]
use fuser::consts::FUSE_HANDLE_KILLPRIV;
#[cfg(feature = "abi-7-31")]
use fuser::consts::FUSE_WRITE_KILL_PRIV;
use fuser::TimeOrNow::Now;
use fuser::{
    Filesystem, KernelConfig, MountOption, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyEmpty, ReplyEntry, ReplyOpen, ReplyStatfs, ReplyWrite, ReplyXattr, Request, TimeOrNow,
    FUSE_ROOT_ID,
};
#[cfg(feature = "abi-7-26")]
use log::info;
use log::{debug, warn};
use log::{error, LevelFilter};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::collections::BTreeMap;
use std::error::Error;
use std::f32::consts::E;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Read, Seek, SeekFrom, Write};
use std::os::raw::c_int;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileExt;
#[cfg(target_os = "linux")]
use std::os::unix::io::IntoRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs, io};
use tribbler::storage::FileRequest;

use async_trait::async_trait;
use log::info;
// use rand;
use std::sync::atomic;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::Receiver;
use tokio::time;
use tokio::time::timeout;

use tribbler::error::{TritonFileError, TritonFileResult, SUCCESS};
use tribbler::{colon, storage};

const BLOCK_SIZE: u64 = 512;
const MAX_NAME_LENGTH: u32 = 255;
const MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024 * 1024;

const ERR_CODE_PLACEHOLDER: i32 = 20;

pub struct Front {
    // original Front
    binstore: Box<dyn storage::BinStorage>,
    clock: atomic::AtomicU64,

    // FS
    data_dir: String,
    next_file_handle: atomic::AtomicU64,
    direct_io: bool,
    suid_support: bool,

    runtime: tokio::runtime::Runtime,
}

impl Front {
    fn new(
        binstore: Box<dyn storage::BinStorage>,
        data_dir: String,
        direct_io: bool,
        #[allow(unused_variables)] suid_support: bool,
    ) -> Front {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        #[cfg(feature = "abi-7-26")]
        {
            Front {
                binstore,
                clock: atomic::AtomicU64::new(1),
                data_dir,
                next_file_handle: AtomicU64::new(1),
                direct_io,
                suid_support,
                runtime,
            }
        }
        #[cfg(not(feature = "abi-7-26"))]
        {
            Front {
                binstore,
                clock: atomic::AtomicU64::new(1),
                data_dir,
                next_file_handle: atomic::AtomicU64::new(1),
                direct_io,
                suid_support: false,
                runtime,
            }
        }
    }

    // Sync my clock to at least at_least, if increment is true,
    // increment my clock to at least at_least. Return the new clock.
    fn clock(&self, at_least: u64, increment: bool) -> TritonFileResult<u64> {
        let my_clock = self.clock.load(atomic::Ordering::SeqCst);
        if my_clock == u64::MAX {
            return Err(Box::new(TritonFileError::MaxedSeq));
        }
        let new_clock = self
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
        Ok(new_clock)
    }
}

impl Filesystem for Front {
    fn lookup(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if name.len() > MAX_NAME_LENGTH as usize {
            return reply.error(libc::ENAMETOOLONG);
        }

        // ReliableStore
        let uid = req.uid().to_string().clone();
        let bin_pre = self.binstore.bin(uid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(mut bin) => {
                let FReq = &FileRequest {
                    uid: req.uid(),
                    gid: req.gid(),
                    pid: req.pid(),
                };
                let bin_lookup_pre = bin.lookup(FReq, parent, name);

                let res = self.runtime.block_on(bin_lookup_pre);

                //FileAttr
                match res {
                    Ok((attrs_op, error_code)) => {
                        if error_code != SUCCESS {
                            reply.error(error_code);
                        } else {
                            let attrs = attrs_op.unwrap();
                            reply.entry(&Duration::new(0, 0), &attrs, 0);
                        }
                    }
                    Err(e) => reply.error(libc::ENETDOWN),
                }
            }
            Err(e) => reply.error(libc::ENETDOWN),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        // ReliableStore
        let uid = _req.uid().to_string().clone();
        let bin_pre = self.binstore.bin(uid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let FReq = &FileRequest {
                    uid: _req.uid(),
                    gid: _req.gid(),
                    pid: _req.pid(),
                };
                let bin_read_pre = bin.read(FReq, inode, fh, offset, size, _flags, _lock_owner);

                let res = self.runtime.block_on(bin_read_pre);

                match res {
                    Ok((string_data_op, error_code)) => {
                        if error_code != SUCCESS {
                            reply.error(error_code);
                        } else {
                            let string_data = string_data_op.unwrap();
                            let str_data: &str = &string_data;
                            let data: Vec<u8> = str_data.as_bytes().to_vec();
                            reply.data(&data)
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn write(
        &mut self,
        _req: &Request,
        inode: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        #[allow(unused_variables)] flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        // ReliableStore
        let FReq = &FileRequest {
            uid: _req.uid(),
            gid: _req.gid(),
            pid: _req.pid(),
        };
        let uid = _req.uid().to_string().clone();
        let bin_pre = self.binstore.bin(uid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(mut bin) => {
                let bin_write_pre = bin.write(
                    FReq,
                    inode,
                    fh,
                    offset,
                    data,
                    _write_flags,
                    flags,
                    _lock_owner,
                );

                let res = self.runtime.block_on(bin_write_pre);

                match res {
                    Ok((written_op, error_code)) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            let written = written_op.unwrap();
                            reply.written(written)
                        }
                    }
                    Err(e) => reply.error(libc::ENETDOWN),
                }
            }
            Err(e) => reply.error(libc::ENETDOWN),
        }
    }

    fn create(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
        mut mode: u32,
        _umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        // ReliableStore
        let FReq = &FileRequest {
            uid: req.uid(),
            gid: req.gid(),
            pid: req.pid(),
        };
        let uid = req.uid().to_string().clone();
        let bin_pre = self.binstore.bin(uid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(mut bin) => {
                let bin_create_pre = bin.create(FReq, parent, name, mode, _umask, flags);

                let res = self.runtime.block_on(bin_create_pre);

                match res {
                    Ok(res_info) => {
                        let (attrs_fh_op, error_code) = res_info;
                        if error_code != SUCCESS {
                            reply.error(error_code);
                        } else {
                            let (attr, fh) = attrs_fh_op.unwrap();
                            reply.created(&Duration::new(0, 0), &attr, 0, fh, 0)
                        }
                    }
                    Err(e) => reply.error(libc::ENETDOWN),
                }
            }
            Err(e) => reply.error(libc::ENETDOWN),
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        // ReliableStore
        let FReq = &FileRequest {
            uid: _req.uid(),
            gid: _req.gid(),
            pid: _req.pid(),
        };
        let uid = _req.uid().to_string().clone();
        let bin_pre = self.binstore.bin(uid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(mut bin) => {
                let bin_unlink_pre = bin.unlink(FReq, parent, name);

                let res = self.runtime.block_on(bin_unlink_pre);

                match res {
                    Ok(error_code) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            reply.ok();
                        }
                    }
                    Err(e) => reply.error(libc::ENETDOWN),
                }
            }
            Err(e) => reply.error(libc::ENETDOWN),
        }
    }
}

pub fn check_access(
    file_uid: u32,
    file_gid: u32,
    file_mode: u16,
    uid: u32,
    gid: u32,
    mut access_mask: i32,
) -> bool {
    // F_OK tests for existence of file
    if access_mask == libc::F_OK {
        return true;
    }
    let file_mode = i32::from(file_mode);

    // root is allowed to read & write anything
    if uid == 0 {
        // root only allowed to exec if one of the X bits is set
        access_mask &= libc::X_OK;
        access_mask -= access_mask & (file_mode >> 6);
        access_mask -= access_mask & (file_mode >> 3);
        access_mask -= access_mask & file_mode;
        return access_mask == 0;
    }

    if uid == file_uid {
        access_mask -= access_mask & (file_mode >> 6);
    } else if gid == file_gid {
        access_mask -= access_mask & (file_mode >> 3);
    } else {
        access_mask -= access_mask & file_mode;
    }

    return access_mask == 0;
}
