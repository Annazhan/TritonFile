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
pub const NON_ERROR:u64 = -1; 

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

impl ServerFileSystem for StorageClient{
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
        // client return string 
        // binstore return string  
        // front: string -> vec[u8]
        let mut client = self.client().await;
        let FReq = disfuser::FRequest{
            uid: _req.uid(),
            gid: _req.gid(),
            pid: _req.pid(),
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
            error_code = item.unwrap().success; 
            if error_code!= NON_ERROR{
                Err(error_code)
            }
        }
        let joined_received = received.join("");
        Ok(joined_received)
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
        //stream writing
        todo!();
        let mut client = self.client().await;
        let result = client
            .write(disfuser::Write{
                _req, 
                inode,
                fh,
                offset,
                data,
                _write_flags,
                flags,
                _lock_owner
            })
            .await?;
        if result.into_inner().errcode!=NON_ERROR{
            Err(result.into_inner().errcode)
        }
        Ok(result.into_inner().size)
    }

    async fn lookup(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<FileAttr>{
        let mut client = self.client().await;
        let stream = client
            .lookup(disfuser::LookUp {
                req, 
                parent,
                name
            })
            .await
            .unwrap()
            .into_inner();
        
        let mut received : Vec<String> = Vec::new();
        let mut error_code: u64; 
        let mut stream = stream.take();
        while let Some(item) = stream.next().await {
            received.push(item.unwrap().message);
            error_code = item.unwrap().success; 
            if error_code!= NON_ERROR{
                Err(error_code)
            }
        }
        let joined_received = received.join("");
        let fileattr = serde_json::from_str::<FileAttr>(joined_received).unwrap();
        Ok(fileattr)
        // receive stream Reply
        // Return FileAttr
        // binstore return FileAttr  
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
        let mut client = self.client().await;
        let result = client
            .create(disfuser::Create {
                req,
                parent,
                name,
                mode,
                _umask, 
                flags
            })
            .await?;
        
        let attr = result.into_inner().attr;
        let fh = result.into_inner().fh;
        let error_code = result.into_inner().errcode;
        if error_code != NON_ERROR{
            Err(error_code);
        }
        let fileattr = serde_json::from_str::<FileAttr>(joined_received).unwrap();
        Ok((fileattr, fh))
        // receive string -> tuple
        // tuple -> fileattr, u64
        // let attr, fh = result.into_inner().attr, result.into_inner().fh; 
        // Ok((attr, fh))
    }

    async fn unlink(&mut self, req: &Request, parent: u64, name: &OsStr) -> TritonFileResult<()>{
        let mut client = self.client().await;
        let result = client
            .unlink(disfuser::Unlink {
                req,
                parent, 
                name
            })
            .await?;
        Ok(())
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
