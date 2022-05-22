use async_trait::async_trait;
use tokio::sync::Mutex;
use tonic::transport::{Channel, Endpoint};
use tonic::Code;
use tribbler::err::TribResult;
use tribbler::rpc;
use tribbler::rpc::trib_storage_client::TribStorageClient;
use tribbler::storage;
use tribbler::storage::{KeyList, KeyString, Storage};

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
    ) -> TritonFileResult<FileStream>{

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
        let mut client = self.client().await;
        let result = client
            .write(rpc::Key {
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

    async fn lookup(
        &mut self,
        req: &Request,
        parent: u64,
        name: &OsStr,
    ) -> TritonFileResult<FileAttr>{

    }

    async fn unlink(&mut self, req: &Request, parent: u64, name: &OsStr) -> TritonFileResult<()>{

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
