use std::cmp::min;
use std::ffi::OsStr;
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use fuser::FileAttr;
use fuser::TimeOrNow;
use libc::c_int;
use std::time::SystemTime;
use tokio::sync::Mutex;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::transport::{Channel, Endpoint};
use tonic::Code;
use tribbler::disfuser::disfuser_client::DisfuserClient;
use tribbler::disfuser::{
    Access, Create, FRequest, Getattr, Getxattr, Listxattr, LookUp, Open, Read, Release, Rename,
    Setxattr, Unlink, Write, Setattr
};
use tribbler::disfuser_server::slice_size;
use tribbler::error::{TritonFileResult, SUCCESS};
use tribbler::rpc;
use tribbler::rpc::trib_storage_client::TribStorageClient;
use tribbler::storage::{self, FileRequest, ServerFileSystem};
use tribbler::storage::{KeyList, KeyString, Storage};

pub const DEFAULT_LOCK_OWNER: u64 = 0;

pub struct StorageClient {
    channel: Mutex<Channel>,
}

pub async fn new_client(addr: &str) -> TritonFileResult<Box<dyn Storage>> {
    Ok(Box::new(StorageClient::new(addr)?))
}

impl StorageClient {
    pub fn new(addr: &str) -> TritonFileResult<StorageClient> {
        let channel = Endpoint::from_shared(format!("http://{}", addr))?.connect_lazy();
        Ok(StorageClient {
            channel: Mutex::new(channel),
        })
    }

    pub async fn client(&self) -> TribStorageClient<Channel> {
        TribStorageClient::new(self.channel.lock().await.clone())
    }

    pub async fn disfuser_client(&self) -> DisfuserClient<Channel> {
        DisfuserClient::new(self.channel.lock().await.clone())
    }
}

// convert the write into a write stream
fn write_requests_iter(
    _req: FRequest,
    inode: u64,
    fh: u64,
    offset: i64,
    data: &[u8],
    _write_flags: u32,
    #[allow(unused_variables)] flags: i32,
    _lock_owner: Option<u64>,
) -> impl Stream<Item = Write> {
    let data_string = serde_json::to_string(data).unwrap();
    let data_len = data_string.len();

    let mut n = data_len / slice_size;
    if data_len % slice_size != 0 {
        n += 1;
    }

    let mut vec: Vec<Write> = Vec::new();
    let mut start: usize;
    let mut end: usize;

    for i in 0..n {
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        start = i * slice_size;
        end = min(start + slice_size, data_len);
        let element = Write {
            frequest: freq,
            ino: inode,
            fh,
            offset,
            data: data_string.clone()[start..end].to_string(),
            write_flag: _write_flags,
            flags,
            lock_owner: _lock_owner,
        };
        vec.push(element);
    }
    return tokio_stream::iter(vec);
}

// convert the write into a write stream
fn setxattr_requests_iter(
    _req: FRequest,
    ino: u64,
    name: &OsStr,
    _value: &[u8],
    flags: i32,
    position: u32,
) -> impl Stream<Item = Setxattr> {
    let data_string = serde_json::to_string(_value).unwrap();
    let data_len = data_string.len();

    let mut n = data_len / slice_size;
    if data_len % slice_size != 0 {
        n += 1;
    }

    let mut vec: Vec<Setxattr> = Vec::new();
    let mut start: usize;
    let mut end: usize;

    let name_string = name.to_str().unwrap().to_string();
    let value_string = serde_json::to_string(_value).unwrap();

    for i in 0..n {
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        start = i * slice_size;
        end = min(start + slice_size, data_len);
        let element = Setxattr {
            frequest: freq,
            ino: ino,
            name: name_string.clone(),
            value: value_string.clone()[start..end].to_string(),
            flags: flags,
            position: position,
        };
        vec.push(element);
    }
    return tokio_stream::iter(vec);
}

#[async_trait]
impl ServerFileSystem for StorageClient {
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
        let mut client = self.disfuser_client().await;
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let mut stream = client
            .read(Read {
                frequest: freq,
                ino: inode,
                fh,
                offset,
                size,
                flags: _flags,
                lock_owner: _lock_owner,
            })
            .await
            .unwrap()
            .into_inner();
        let mut received: Vec<String> = Vec::new();
        let mut error_code: c_int;
        while let Some(item) = stream.next().await {
            let reply = item.unwrap();
            received.push(reply.message);
            error_code = reply.errcode;
            if error_code != SUCCESS {
                return Ok((None, error_code));
            }
        }
        let joined_received = received.join("");
        Ok((Some(joined_received), SUCCESS))
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
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let mut client = self.disfuser_client().await;

        let in_stream = write_requests_iter(
            freq,
            inode,
            fh,
            offset,
            data,
            _write_flags,
            flags,
            _lock_owner,
        );

        let result = client.write(in_stream).await?;

        let write_reply = result.into_inner();
        let size = write_reply.size;
        let error_code = write_reply.errcode;
        if error_code != SUCCESS {
            return Ok((None, error_code));
        }
        Ok((Some(size), SUCCESS))
    }

    async fn lookup(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)> {
        let freq = FRequest {
            uid: req.uid,
            gid: req.gid,
            pid: req.pid,
        };

        let name_string = name.to_str().unwrap().to_string();

        let mut client = self.disfuser_client().await;
        let result = client
            .lookup(LookUp {
                frequest: freq,
                parent: parent,
                name: name_string,
            })
            .await?;
        let lookup_reply = result.into_inner();
        let error_code = lookup_reply.errcode;
        if error_code != SUCCESS {
            return Ok((None, error_code));
        }
        let received_attr = lookup_reply.message;
        let fileattr = serde_json::from_str::<FileAttr>(&received_attr).unwrap();
        Ok((Some(fileattr), SUCCESS))
    }

    async fn create(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        flags: i32,
    ) -> TritonFileResult<(Option<(FileAttr, u64)>, c_int)> {
        let freq = FRequest {
            uid: req.uid,
            gid: req.gid,
            pid: req.pid,
        };

        let name_string = name.to_str().unwrap().to_string();
        let mut client = self.disfuser_client().await;
        let result = client
            .create(Create {
                frequest: freq,
                parent: parent,
                name: name_string,
                mode: mode,
                umask: _umask,
                flags: flags,
            })
            .await?;

        let create_reply = result.into_inner();
        let attr = create_reply.file_attr;
        let fh = create_reply.fh;
        let error_code = create_reply.errcode;
        if error_code != SUCCESS {
            return Ok((None, error_code));
        }
        let fileattr = serde_json::from_str::<FileAttr>(&attr).unwrap();
        Ok((Some((fileattr, fh)), SUCCESS))
    }

    async fn unlink(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<c_int> {
        let freq = FRequest {
            uid: req.uid,
            gid: req.gid,
            pid: req.pid,
        };

        let name_string = name.to_str().unwrap().to_string();
        let mut client = self.disfuser_client().await;
        let result = client
            .unlink(Unlink {
                frequest: freq,
                parent: parent,
                name: name_string,
            })
            .await?;
        let error_code = result.into_inner().errcode;
        if error_code != SUCCESS {
            return Ok(error_code);
        }
        Ok(SUCCESS)
    }

    async fn getattr(
        &self,
        _req: &FileRequest,
        ino: u64,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)> {
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let mut client = self.disfuser_client().await;
        let result = client
            .getattr(Getattr {
                frequest: freq,
                ino: ino,
            })
            .await?;
        let getattr_reply = result.into_inner();
        let attr = getattr_reply.file_attr;
        let error_code = getattr_reply.errcode;
        if error_code != SUCCESS {
            return Ok((None, error_code));
        }
        let fileattr = serde_json::from_str::<FileAttr>(&attr).unwrap();
        Ok((Some(fileattr), SUCCESS))
    }

    async fn open(
        &self,
        _req: &FileRequest,
        _ino: u64,
        _flags: i32,
    ) -> TritonFileResult<(Option<(u64, u32)>, c_int)> {
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let mut client = self.disfuser_client().await;
        let result = client
            .open(Open {
                frequest: freq,
                ino: _ino,
                flags: _flags,
            })
            .await?;
        let open_reply = result.into_inner();
        let fh = open_reply.fh;
        let open_flag = open_reply.openflag;
        let error_code = open_reply.errcode;
        if error_code != SUCCESS {
            return Ok((None, error_code));
        }
        Ok((Some((fh, open_flag)), SUCCESS))
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
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let mut client = self.disfuser_client().await;
        let result = client
            .release(Release {
                frequest: freq,
                ino: _ino,
                fh: _fh,
                flags: _flags,
                lock_owner: _lock_owner,
                flush: _flush,
            })
            .await?;
        let error_code = result.into_inner().errcode;
        if error_code != SUCCESS {
            return Ok(error_code);
        }
        Ok(SUCCESS)
    }

    async fn setxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        name: &OsStr,
        _value: &[u8],
        flags: i32,
        position: u32,
    ) -> TritonFileResult<c_int> {
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };
        let mut client = self.disfuser_client().await;
        let in_stream = setxattr_requests_iter(
            freq,
            ino,
            name,
            _value,
            flags,
            position,
        );

        let result = client.setxattr(in_stream).await?;
        let error_code = result.into_inner().errcode;
        if error_code != SUCCESS {
            return Ok(error_code);
        }
        Ok(SUCCESS)
    }

    //reply Vec<u8> as string
    async fn getxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        name: &OsStr,
        size: u32,
    ) -> TritonFileResult<(Option<(String, u32)>, c_int)> {
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let name_string = name.to_str().unwrap().to_string();
        let mut client = self.disfuser_client().await;
        let mut stream = client
            .getxattr(Getxattr {
                frequest: freq,
                ino: ino,
                name: name_string,
                size: size,
            })
            .await.unwrap().into_inner();

        let mut received: Vec<String> = Vec::new();
        let mut error_code: c_int;
        let mut size: u32 = 0; 
        while let Some(item) = stream.next().await {
            let reply = item.unwrap();
            received.push(reply.data);
            size = reply.size; 
            error_code = reply.errcode;
            if error_code != SUCCESS {
                return Ok((None, error_code));
            }
        }
        let joined_received = received.join("");
        Ok((Some((joined_received, size)), SUCCESS))
    }

    async fn listxattr(
        &self,
        _req: &FileRequest,
        ino: u64,
        size: u32,
    ) -> TritonFileResult<(Option<(String, u32)>, c_int)> {
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let mut client = self.disfuser_client().await;
        
        let mut stream = client
        .listxattr(Listxattr {
            frequest: freq,
            ino: ino,
            size: size,
        })
        .await.unwrap().into_inner();

        let mut received: Vec<String> = Vec::new();
        let mut error_code: c_int;
        let mut size: u32 = 0;
        while let Some(item) = stream.next().await {
            let reply = item.unwrap();
            received.push(reply.data);
            size = reply.size; 
            error_code = reply.errcode;
            if error_code != SUCCESS {
                return Ok((None, error_code));
            }
        }
        let joined_received = received.join("");
        Ok((Some((joined_received, size)), SUCCESS))

    }

    async fn access(&self, _req: &FileRequest, ino: u64, mask: i32) -> TritonFileResult<c_int> {
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };
        let mut client = self.disfuser_client().await;
        let result = client
            .access(Access {
                frequest: freq,
                ino: ino,
                mask: mask,
            })
            .await?;
        let error_code = result.into_inner().errcode;
        if error_code != SUCCESS {
            return Ok(error_code);
        }
        Ok(SUCCESS)
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
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };
        let name_string = name.to_str().unwrap().to_string();
        let newname_string = newname.to_str().unwrap().to_string();
        let mut client = self.disfuser_client().await;
        let result = client
            .rename(Rename {
                frequest: freq,
                parent: parent,
                name: name_string,
                newparent: newparent,
                newname: newname_string,
                flags: flags,
            })
            .await?;
        let error_code = result.into_inner().errcode;
        if error_code != SUCCESS {
            return Ok(error_code);
        }
        Ok(SUCCESS)
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
        let freq = FRequest {
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let atime_secs: Option<i64>; 
        let atime_nsecs: Option<u32>; 
        (atime_secs, atime_nsecs) = match _atime{
            Some(timeornow) => match timeornow{
                TimeOrNow::SpecificTime(system_time) =>{
                    match system_time.duration_since(UNIX_EPOCH) {
                        Ok(duration) => (Some(duration.as_secs() as i64), Some(duration.subsec_nanos())),
                        Err(before_epoch_error) => (
                            Some(-(before_epoch_error.duration().as_secs() as i64)),
                            Some(before_epoch_error.duration().subsec_nanos()),
                        ),
                    }
                }
                TimeOrNow::Now =>{
                    let system_time = SystemTime::now(); 
                    match system_time.duration_since(UNIX_EPOCH) {
                        Ok(duration) => (Some(duration.as_secs() as i64), Some(duration.subsec_nanos())),
                        Err(before_epoch_error) => (
                            Some(-(before_epoch_error.duration().as_secs() as i64)),
                            Some(before_epoch_error.duration().subsec_nanos()),
                        ),
                    }
                }
            }
            None => (None, None)
        };

        let mtime_secs: Option<i64>; 
        let mtime_nsecs: Option<u32>; 
        (mtime_secs, mtime_nsecs) = match _mtime{
            Some(timeornow) => match timeornow{
                TimeOrNow::SpecificTime(system_time) =>{
                    match system_time.duration_since(UNIX_EPOCH) {
                        Ok(duration) => (Some(duration.as_secs() as i64), Some(duration.subsec_nanos())),
                        Err(before_epoch_error) => (
                            Some(-(before_epoch_error.duration().as_secs() as i64)),
                            Some(before_epoch_error.duration().subsec_nanos()),
                        ),
                    }
                }
                TimeOrNow::Now =>{
                    let system_time = SystemTime::now(); 
                    match system_time.duration_since(UNIX_EPOCH) {
                        Ok(duration) => (Some(duration.as_secs() as i64), Some(duration.subsec_nanos())),
                        Err(before_epoch_error) => (
                            Some(-(before_epoch_error.duration().as_secs() as i64)),
                            Some(before_epoch_error.duration().subsec_nanos()),
                        ),
                    }
                }
            }
            None => (None, None)
        };

        let mut client = self.disfuser_client().await;
        let result = client
            .setattr(Setattr {
                frequest: freq,
                ino: ino,
                mode: mode,
                uid: uid,
                gid: gid,
                size: size,
                fh: fh,
                flags: flags,
                atime_secs, 
                atime_nsecs, 
                mtime_secs, 
                mtime_nsecs 
            })
            .await?;

        let setattr_reply = result.into_inner();
        let error_code = setattr_reply.errcode;
        if error_code != SUCCESS {
            return Ok((None, error_code));
        }
        let received_attr = setattr_reply.file_attr;
        let fileattr = serde_json::from_str::<FileAttr>(&received_attr).unwrap();
        Ok((Some(fileattr), SUCCESS))
    }
}

#[async_trait]
impl KeyString for StorageClient {
    async fn get(&self, key: &str) -> TritonFileResult<Option<String>> {
        let mut client = self.client().await;

        let result = client
            .get(rpc::Key {
                key: key.to_string(),
            })
            .await;
        match result {
            Ok(val) => Ok(Some(val.into_inner().value)),
            Err(err) => {
                if err.code() == Code::NotFound {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    async fn set(&self, kv: &storage::KeyValue) -> TritonFileResult<bool> {
        let mut client = self.client().await;
        let result = client
            .set(rpc::KeyValue {
                key: kv.key.clone(),
                value: kv.value.clone(),
            })
            .await?;
        Ok(result.into_inner().value)
    }
    async fn keys(&self, p: &storage::Pattern) -> TritonFileResult<storage::List> {
        let mut client = self.client().await;
        let result = client
            .keys(rpc::Pattern {
                prefix: p.prefix.clone(),
                suffix: p.suffix.clone(),
            })
            .await?;
        Ok(storage::List(result.into_inner().list))
    }
}

#[async_trait]
impl KeyList for StorageClient {
    async fn list_get(&self, key: &str) -> TritonFileResult<storage::List> {
        let mut client = self.client().await;
        let result = client
            .list_get(rpc::Key {
                key: key.to_string(),
            })
            .await?;
        Ok(storage::List(result.into_inner().list))
    }

    async fn list_keys(&self, p: &storage::Pattern) -> TritonFileResult<storage::List> {
        let mut client = self.client().await;
        let result = client
            .list_keys(rpc::Pattern {
                prefix: p.prefix.to_string(),
                suffix: p.suffix.to_string(),
            })
            .await?;
        Ok(storage::List(result.into_inner().list))
    }

    async fn list_append(&self, kv: &storage::KeyValue) -> TritonFileResult<bool> {
        let mut client = self.client().await;
        let result = client
            .list_append(rpc::KeyValue {
                key: kv.key.clone(),
                value: kv.value.clone(),
            })
            .await?;
        Ok(result.into_inner().value)
    }

    async fn list_remove(&self, kv: &storage::KeyValue) -> TritonFileResult<u32> {
        let mut client = self.client().await;
        let result = client
            .list_remove(rpc::KeyValue {
                key: kv.key.clone(),
                value: kv.value.clone(),
            })
            .await?;
        Ok(result.into_inner().removed)
    }
}

#[async_trait]
impl Storage for StorageClient {
    async fn clock(&self, at_least: u64) -> TritonFileResult<u64> {
        let mut client = self.client().await;
        let result = client
            .clock(rpc::Clock {
                timestamp: at_least,
            })
            .await?;
        Ok(result.into_inner().timestamp)
    }
}
