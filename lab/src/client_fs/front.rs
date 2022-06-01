#![allow(clippy::needless_return)]

#[cfg(feature = "abi-7-26")]
use fuser::consts::FUSE_HANDLE_KILLPRIV;
#[cfg(feature = "abi-7-31")]
use fuser::consts::FUSE_WRITE_KILL_PRIV;
use fuser::{Filesystem, ReplyCreate, ReplyData, ReplyEmpty, ReplyEntry, ReplyWrite, Request, TimeOrNow, ReplyAttr, ReplyXattr, ReplyOpen, KernelConfig, ReplyStatfs};
use libc::c_int;
use log::info;
#[cfg(feature = "abi-7-26")]
use log::info;
use std::ffi::OsStr;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicU64, Ordering};
use tribbler::storage::FileRequest;
use std::time::{Duration, SystemTime};
use std::sync::atomic;

use tribbler::error::{TritonFileError, TritonFileResult, SUCCESS};
use tribbler::storage;

const BLOCK_SIZE: u64 = 512;
const MAX_NAME_LENGTH: u32 = 255;
const MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024 * 1024;

pub struct Front {
    binstore: Box<dyn storage::BinStorage>,
    clock: atomic::AtomicU64,
    runtime: tokio::runtime::Runtime,
}

impl Front {
    pub fn new(binstore: Box<dyn storage::BinStorage>, runtime: tokio::runtime::Runtime) -> Front {
        #[cfg(feature = "abi-7-26")]
        {
            Front {
                binstore,
                clock: atomic::AtomicU64::new(1),
                runtime,
            }
        }
        #[cfg(not(feature = "abi-7-26"))]
        {
            Front {
                binstore,
                clock: atomic::AtomicU64::new(1),
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
    fn init(
        &mut self,
        _req: &Request,
        #[allow(unused_variables)] config: &mut KernelConfig,
    ) -> Result<(), c_int> {
        info!("front init function");
        #[cfg(feature = "abi-7-26")]
        config.add_capabilities(FUSE_HANDLE_KILLPRIV).unwrap();
        
        // ReliableStore
        let gid = _req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let freq = &FileRequest {
                    uid: _req.uid(),
                    gid: _req.gid(),
                    pid: _req.pid(),
                };
                let bin_init_pre = bin.init(freq);

                let res = self.runtime.block_on(bin_init_pre);
                match res {
                    Ok(error_code) => {
                        if error_code != SUCCESS {
                            return Err(error_code)
                        } else {
                            return Ok(())
                        }
                    }
                    Err(_) => return Err(libc::ENETDOWN),
                }
            }
            Err(_) => return Err(libc::EACCES)
        }
        Ok(())
    }

    fn lookup(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        info!("front  front Look up function");
        // ReliableStore
        let gid = req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let freq = &FileRequest {
                    uid: req.uid(),
                    gid: req.gid(),
                    pid: req.pid(),
                };
                let bin_lookup_pre = bin.lookup(freq, parent, name);

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
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn forget(&mut self, _req: &Request, _ino: u64, _nlookup: u64) {}

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
        info!("front  front read function {}", inode);
        // ReliableStore
        let gid = _req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let freq = &FileRequest {
                    uid: _req.uid(),
                    gid: _req.gid(),
                    pid: _req.pid(),
                };
                let bin_read_pre = bin.read(freq, inode, fh, offset, size, _flags, _lock_owner);

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
        info!("front  front write {}", inode);
        // ReliableStore
        let freq = &FileRequest {
            uid: _req.uid(),
            gid: _req.gid(),
            pid: _req.pid(),
        };
        let gid = _req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_write_pre = bin.write(
                    freq,
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
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn create(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        info!("front create function {}", parent);
        let freq = &FileRequest {
            uid: req.uid(),
            gid: req.gid(),
            pid: req.pid(),
        };
        let gid = req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_create_pre = bin.create(freq, parent, name, mode, _umask, flags);

                let res = self.runtime.block_on(bin_create_pre);

                match res {
                    Ok(res_info) => {
                        let (attrs_fh_op, error_code) = res_info;
                        if error_code != SUCCESS {
                            reply.error(error_code);
                        } else {
                            let (attrs, fh) = attrs_fh_op.unwrap();
                            reply.created(&Duration::new(0, 0), &attrs, 0, fh, 0)
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        info!("front  unlink function {}", parent);
        let freq = &FileRequest {
            uid: _req.uid(),
            gid: _req.gid(),
            pid: _req.pid(),
        };
        let gid = _req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_unlink_pre = bin.unlink(freq, parent, name);

                let res = self.runtime.block_on(bin_unlink_pre);

                match res {
                    Ok(error_code) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            reply.ok();
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn getattr(&mut self, _req: &Request, inode: u64, reply: ReplyAttr) {
        info!("front  get attr function {}", inode);
        let freq = &FileRequest {
            uid: _req.uid(),
            gid: _req.gid(),
            pid: _req.pid(),
        };
        let gid = _req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_getattr_pre = bin.getattr(freq, inode);

                let res = self.runtime.block_on(bin_getattr_pre);

                match res {
                    Ok(res_info) => {
                        let (attrs_op, error_code) = res_info;
                        if error_code != SUCCESS {
                            reply.error(error_code);
                        } else {
                            let attrs = attrs_op.unwrap();
                            reply.attr(&Duration::new(0, 0), &attrs);
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn open(&mut self, req: &Request, inode: u64, flags: i32, reply: ReplyOpen) {
        info!("front  open function {}", inode);
        let freq = &FileRequest {
            uid: req.uid(),
            gid: req.gid(),
            pid: req.pid(),
        };
        let gid = req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_open_pre = bin.open(freq, inode, flags);

                let res = self.runtime.block_on(bin_open_pre);

                match res {
                    Ok(res_info) => {
                        let (attrs_op, error_code) = res_info;
                        if error_code != SUCCESS {
                            reply.error(error_code);
                        } else {
                            let (fh, open_flags) = attrs_op.unwrap();
                            reply.opened(fh, open_flags);
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn release(
        &mut self,
        _req: &Request<'_>,
        inode: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        info!("front  release {}", inode);
        let freq = &FileRequest {
            uid: _req.uid(),
            gid: _req.gid(),
            pid: _req.pid(),
        };
        let gid = _req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_release_pre = bin.release(freq, inode, _fh, _flags, _lock_owner, _flush);

                let res = self.runtime.block_on(bin_release_pre);

                match res {
                    Ok(error_code) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            reply.ok();
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn statfs(&mut self, _req: &Request, _ino: u64, reply: ReplyStatfs) {
        info!("statfs() implementation is a stub");
        // TODO: real implementation of this
        reply.statfs(
            10_000,
            10_000,
            10_000,
            1,
            10_000,
            BLOCK_SIZE as u32,
            MAX_NAME_LENGTH,
            BLOCK_SIZE as u32,
        );
    }

    fn setxattr(
        &mut self,
        request: &Request<'_>,
        inode: u64,
        key: &OsStr,
        value: &[u8],
        _flags: i32,
        _position: u32,
        reply: ReplyEmpty,
    ) {
        info!("front  set x attr {}", inode);
        let freq = &FileRequest {
            uid: request.uid(),
            gid: request.gid(),
            pid: request.pid(),
        };
        let gid = request.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_setxattr_pre = bin.setxattr(freq, inode, key, value, _flags, _position);

                let res = self.runtime.block_on(bin_setxattr_pre);

                match res {
                    Ok(error_code) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            reply.ok();
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn getxattr(
        &mut self,
        request: &Request<'_>,
        inode: u64,
        key: &OsStr,
        size: u32,
        reply: ReplyXattr,
    ) {
        info!("front  get x attr {}", inode);
        let freq = &FileRequest {
            uid: request.uid(),
            gid: request.gid(),
            pid: request.pid(),
        };
        let gid = request.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_getxattr_pre = bin.getxattr(freq, inode, key, size);
                let res = self.runtime.block_on(bin_getxattr_pre);

                match res {
                    Ok((data_op, error_code)) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            let (string_data, data_len) = data_op.unwrap();
                            let str_data: &str = &string_data;
                            let data: Vec<u8> = str_data.as_bytes().to_vec();

                            if size == 0 {
                                reply.size(data_len);
                            } else if data_len <= size {
                                reply.data(&data);
                            } 
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn listxattr(&mut self, _req: &Request<'_>, inode: u64, size: u32, reply: ReplyXattr) {
        info!("front  list x attr {}", inode);
        let freq = &FileRequest {
            uid: _req.uid(),
            gid: _req.gid(),
            pid: _req.pid(),
        };
        let gid = _req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_listxattr_pre = bin.listxattr(freq, inode, size);
                let res = self.runtime.block_on(bin_listxattr_pre);

                match res {
                    Ok((data_op, error_code)) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            let (string_data, data_len) = data_op.unwrap();
                            let str_data: &str = &string_data;
                            let data: Vec<u8> = str_data.as_bytes().to_vec();

                            if size == 0 {
                                reply.size(data_len);
                            } else if data_len <= size {
                                reply.data(&data);
                            } 
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn access(&mut self, req: &Request, inode: u64, mask: i32, reply: ReplyEmpty) {
        info!("front  access {}", inode);
        let freq = &FileRequest {
            uid: req.uid(),
            gid: req.gid(),
            pid: req.pid(),
        };
        let gid = req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);
        
        info!("front  access: get bin_res");
        match bin_res {
            Ok(bin) => {
                let bin_access_pre = bin.access(freq, inode, mask);

                let res = self.runtime.block_on(bin_access_pre);

                match res {
                    Ok(error_code) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            reply.ok();
                        }
                    }
                    Err(e) => {
                        info!("access error 1 {}", e); 
                        reply.error(libc::ENETDOWN)
                    },
                }
            }
            Err(e) => {
                info!("access error 2 {}", e); 
                reply.error(libc::ENETDOWN)
            },
        }
    }

    fn rename(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
        new_parent: u64,
        new_name: &OsStr,
        flags: u32,
        reply: ReplyEmpty,
    ) {
        info!("front rename function {}", parent);
        let freq = &FileRequest {
            uid: req.uid(),
            gid: req.gid(),
            pid: req.pid(),
        };
        let gid = req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_rename_pre = bin.rename(freq, parent, name, new_parent, new_name, flags);

                let res = self.runtime.block_on(bin_rename_pre);

                match res {
                    Ok(error_code) => {
                        if error_code != SUCCESS {
                            reply.error(error_code)
                        } else {
                            reply.ok();
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }

    fn setattr(
        &mut self,
        req: &Request,
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
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        info!("front set attr {}", inode);
        let freq = &FileRequest {
            uid: req.uid(),
            gid: req.gid(),
            pid: req.pid(),
        };
        let gid_string = req.gid().to_string().clone();
        let bin_pre = self.binstore.bin(gid_string.as_str());
        let bin_res = self.runtime.block_on(bin_pre);

        match bin_res {
            Ok(bin) => {
                let bin_setattr_pre = bin.setattr(
                    freq, inode, mode, uid, gid, size, atime, mtime, _ctime, fh, _crtime, _chgtime,
                    _bkuptime, _flags,
                );

                let res = self.runtime.block_on(bin_setattr_pre);

                match res {
                    Ok(res_info) => {
                        let (attrs_op, error_code) = res_info;
                        if error_code != SUCCESS {
                            reply.error(error_code);
                        } else {
                            let attrs = attrs_op.unwrap();
                            reply.attr(&Duration::new(0, 0), &attrs);
                        }
                    }
                    Err(_) => reply.error(libc::ENETDOWN),
                }
            }
            Err(_) => reply.error(libc::ENETDOWN),
        }
    }
}