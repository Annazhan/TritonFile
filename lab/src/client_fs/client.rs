use async_trait::async_trait;
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint};
use tonic::Code;
use tribbler::err::TribResult;
use tribbler::rpc;
use tribbler::rpc::trib_storage_client::TribStorageClient;
use tribbler::storage;
use tribbler::storage::{KeyList, KeyString, Storage};
use tribbler::disfuser;

pub const slice_size: usize = 1024; 
pub struct StorageClient {
    channel: Mutex<Channel>,
}

pub async fn new_client(addr: &str) -> TribResult<Box<dyn Storage>> {
    Ok(Box::new(StorageClient::new(addr)?))
}

impl StorageClient {
    pub fn new(addr: &str) -> TribResult<StorageClient> {
        let channel = Endpoint::from_shared(format!("http://{}", addr))?.connect_lazy();
        Ok(StorageClient {
            channel: Mutex::new(channel),
        })
    }

    pub async fn client(&self) -> TribStorageClient<Channel> {
        TribStorageClient::new(self.channel.lock().await.clone())
    }
}

// convert the write into a write stream 
fn write_requests_iter(
    _req: &FileRequest,
    inode: u64,
    fh: u64,
    offset: i64,
    data: &[u8],
    _write_flags: u32,
    #[allow(unused_variables)] flags: i32,
    _lock_owner: Option<u64>, 
) -> impl Stream<Item = Write> {
    let FReq = disfuser::FRequest{
        uid: _req.uid,
        gid: _req.gid,
        pid: _req.pid,
    };

    let data_string = serde_json::to_string(data); 
    let data_len = data_string.len();
    
    let mut n = data_len/slice_size; 
    if data_len % slice_size!=0{
        n += 1; 
    }

    let mut vec: Vec<disfuser::Write> = Vec::new();
    let mut start = 0; 
    let mut end = 0;

    for i in 0..n {
        start = i * slice_size; 
        end = min(start + slice_size, data_len);
        let element = disfuser::Write{
            FReq, 
            inode,
            fh,
            offset,
            data_string.clone()[start..end].to_string(),
            _write_flags,
            flags,
            _lock_owner
        }; 
        vec.push(element); 
    }

    return Stream::iter(vec)
}

impl ServerFileSystem for StorageClient{
    async fn read(
        &self,
        _req: &FileRequest,
        inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
    ) -> TritonFileResult<(Option<String>, c_int)>{
        let mut client = self.client().await;
        let FReq = disfuser::FRequest{
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let stream = client
            .read(disfuser::Read{
                FReq, 
                inode,
                fh,
                offset,
                size,
                _flags,
                _lock_owner
            }).await
            .unwrap()
            .into_inner();
        let mut received : Vec<String> = Vec::new();
        let mut error_code: u64; 
        let mut stream = stream.take();
        while let Some(item) = stream.next().await {
            received.push(item.unwrap().message);
            error_code = item.unwrap().errcode; 
            if error_code!=SUCCESS{
                Ok((None, error_code))
            }
        }
        let joined_received = received.join("");
        Ok((joined_received, SUCCESS))
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
    ) -> TritonFileResult<(Option<u32>, c_int)>{
        let FReq = disfuser::FRequest{
            uid: _req.uid,
            gid: _req.gid,
            pid: _req.pid,
        };

        let mut client = self.client().await;
                
        let in_stream = write_requests_iter(
            FReq, 
            inode,
            fh,
            offset,
            data,
            _write_flags,
            flags,
            _lock_owner
        );

        let result = client
            .write(in_stream)
            .await?;
        
        let error_code = result.into_inner().errcode;
        if error_code!=SUCCESS{
            Ok((None, error_code))
        }
        Ok((result.into_inner().size, SUCCESS))
    }

    async fn lookup(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<(Option<FileAttr>, c_int)>{
        let FReq = disfuser::FRequest{
            uid: req.uid,
            gid: req.gid,
            pid: req.pid,
        };

        let mut client = self.client().await;
        let result = client
            .lookup(disfuser::LookUp {
                Freq, 
                parent,
                name
            })
        .await?;
        let error_code = result.into_inner().errcode;
        if error_code!=SUCCESS{
            Ok((None, error_code))
        }
        let received_attr = result.into_inner().message;
        let fileattr = serde_json::from_str::<FileAttr>(received_attr).unwrap();
        Ok((fileattr, SUCCESS))
    }

    async fn create(
        &self,
        req: &FileRequest,
        parent: u64,
        name: &OsStr,
        mut mode: u32,
        _umask: u32,
        flags: i32,
    ) -> TritonFileResult<(Option<(FileAttr, u64)>, c_int)>{
        let FReq = disfuser::FRequest{
            uid: req.uid,
            gid: req.gid,
            pid: req.pid,
        };

        let mut client = self.client().await;
        let result = client
            .create(disfuser::Create {
                FReq,
                parent,
                name,
                mode,
                _umask, 
                flags
            })
            .await?;
        
        let attr = result.into_inner().fileAttr;
        let fh = result.into_inner().fh;
        let error_code = result.into_inner().errcode;
        if error_code!=SUCCESS{
            Ok((None, error_code))
        }
        let fileattr = serde_json::from_str::<FileAttr>(joined_received).unwrap();
        Ok(((fileattr, fh), SUCCESS))
        // receive string -> tuple
        // tuple -> fileattr, u64
        // let attr, fh = result.into_inner().attr, result.into_inner().fh; 
        // Ok((attr, fh))
    }

    fn unlink(&self, req: &FileRequest, parent: u64, name: &OsStr) -> TritonFileResult<c_int>{
        let FReq = disfuser::FRequest{
            uid: req.uid,
            gid: req.gid,
            pid: req.pid,
        };

        let mut client = self.client().await;
        let result = client
            .unlink(disfuser::Unlink {
                FReq,
                parent, 
                name
            })
            .await?;
        let error_code = result.into_inner().errcode;
        if error_code!=SUCCESS{
            Ok(error_code)
        }
        Ok(SUCCESS)
    }
}

#[async_trait]
impl KeyString for StorageClient {
    async fn get(&self, key: &str) -> TribResult<Option<String>> {
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

    async fn set(&self, kv: &storage::KeyValue) -> TribResult<bool> {
        let mut client = self.client().await;
        let result = client
            .set(rpc::KeyValue {
                key: kv.key.clone(),
                value: kv.value.clone(),
            })
            .await?;
        Ok(result.into_inner().value)
    }
    async fn keys(&self, p: &storage::Pattern) -> TribResult<storage::List> {
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
    async fn list_get(&self, key: &str) -> TribResult<storage::List> {
        let mut client = self.client().await;
        let result = client
            .list_get(rpc::Key {
                key: key.to_string(),
            })
            .await?;
        Ok(storage::List(result.into_inner().list))
    }

    async fn list_keys(&self, p: &storage::Pattern) -> TribResult<storage::List> {
        let mut client = self.client().await;
        let result = client
            .list_keys(rpc::Pattern {
                prefix: p.prefix.to_string(),
                suffix: p.suffix.to_string(),
            })
            .await?;
        Ok(storage::List(result.into_inner().list))
    }

    async fn list_append(&self, kv: &storage::KeyValue) -> TribResult<bool> {
        let mut client = self.client().await;
        let result = client
            .list_append(rpc::KeyValue {
                key: kv.key.clone(),
                value: kv.value.clone(),
            })
            .await?;
        Ok(result.into_inner().value)
    }

    async fn list_remove(&self, kv: &storage::KeyValue) -> TribResult<u32> {
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
    async fn clock(&self, at_least: u64) -> TribResult<u64> {
        let mut client = self.client().await;
        let result = client
            .clock(rpc::Clock {
                timestamp: at_least,
            })
            .await?;
        Ok(result.into_inner().timestamp)
    }
} 