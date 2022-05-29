use crate::disfuser::disfuser_server::{self, Disfuser};
use crate::disfuser::{
    Access, AccessReply, Create, CreateReply, Getattr, GetattrReply, Getxattr, GetxattrReply,
    Listxattr, ListxattrReply, LookUp, Open, OpenReply, Read, Release, ReleaseReply, Rename,
    RenameReply, Reply, Setattr, SetattrReply, Setxattr, SetxattrReply, Unlink, UnlinkReply, Write,
    WriteReply,
};
use crate::error::SUCCESS;
use crate::storage::FileRequest;
use crate::storage::Storage;
use async_trait::async_trait;
use fuser::{BackgroundSession, FileAttr, MountOption, Request};
use std::cmp::min;
use std::ffi::OsString;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::{Response, Status};
type readStream = Pin<Box<dyn Stream<Item = Result<Reply, Status>> + Send>>;
type getxattrStream = Pin<Box<dyn Stream<Item = Result<GetxattrReply, Status>> + Send>>;
type listxattrStream = Pin<Box<dyn Stream<Item = Result<ListxattrReply, Status>> + Send>>;
// type readStream = Pin<Box<dyn Stream<Item = Result<Read, Status>> + Send>>;
// type lookupStream = Pin<Box<dyn Stream<Item = Result<LookUp, Status>> + Send>>;
pub const slice_size: usize = 128;
use crate::storage;

pub struct DisfuserServer {
    pub filesystem: Box<dyn Storage>,
    // pub clock: RwLock<i64>,
}

fn reply_response_iter(msg: String, errcode: i32) -> Vec<Reply> {
    let data_len = msg.len();

    let mut n = data_len / slice_size;
    if data_len % slice_size != 0 {
        n += 1;
    }

    let mut vec: Vec<Reply> = Vec::new();
    let mut start = 0;
    let mut end = 0;

    for i in 0..n {
        start = i * slice_size;
        end = min(start + slice_size, data_len);
        let element = Reply {
            message: msg.clone()[start..end].to_string(),
            errcode: errcode,
        };
        vec.push(element.clone());
    }
    return vec;
}

fn getxattr_response_iter(msg: String, size: u32, errcode: i32) -> Vec<GetxattrReply> {
    let data_len = msg.len();
    let mut n = data_len / slice_size;
    if data_len % slice_size != 0 {
        n += 1;
    }

    let mut vec: Vec<GetxattrReply> = Vec::new();
    let mut start = 0;
    let mut end = 0;

    for i in 0..n {
        start = i * slice_size;
        end = min(start + slice_size, data_len);
        let element = GetxattrReply {
            data: msg.clone()[start..end].to_string(),
            size: size,
            errcode: errcode,
        };
        vec.push(element.clone());
    }
    return vec;
}

impl DisfuserServer {
    pub fn new(storage: Box<dyn storage::Storage>) -> DisfuserServer {
        DisfuserServer {
            filesystem: storage,
        }
    }
}

#[async_trait]
impl Disfuser for DisfuserServer {
    type readStream = readStream;
    type getxattrStream = getxattrStream;
    type listxattrStream = listxattrStream;

    async fn read(
        &self,
        request: tonic::Request<Read>,
    ) -> Result<tonic::Response<Self::readStream>, tonic::Status> {
        // unwrap all the input Read into different values
        // put unwrapped value into ServerFileSystem read function
        // get read result, stringify it
        // and then stream it and send back to client
        let r_inner = request.into_inner();

        let request = FileRequest {
            uid: r_inner.frequest.clone().uid,
            gid: r_inner.frequest.clone().gid,
            pid: r_inner.frequest.clone().pid,
        };

        let result = self
            .filesystem
            .read(
                &request,
                r_inner.ino,
                r_inner.fh,
                r_inner.offset,
                r_inner.size,
                r_inner.flags,
                r_inner.lock_owner,
            )
            .await;
        let mut reply = Vec::new();
        match result {
            Ok(value) => {
                if value.1 != SUCCESS {
                    reply = vec![Reply {
                        message: "".to_string(),
                        errcode: value.1,
                    }];
                } else {
                    // divide the message into appropriate size, and make a vector
                    let content = value.0.unwrap();
                    reply = reply_response_iter(content, SUCCESS);
                }
            }
            Err(_) => {}
        };

        let mut stream = Box::pin(tokio_stream::iter(reply).throttle(Duration::from_millis(200)));
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                        // item (server response) was queued to be send to client
                    }
                    Err(_item) => {
                        // output_stream was build from rx and both are dropped
                        break;
                    }
                }
            }
            println!("\tclient disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::readStream))
    }

    async fn write(
        &self,
        request: tonic::Request<tonic::Streaming<Write>>,
    ) -> Result<tonic::Response<WriteReply>, tonic::Status> {
        let mut in_stream = request.into_inner();

        let mut r_data: Vec<String> = Vec::new();
        let mut file_request: FileRequest = FileRequest {
            uid: 0,
            gid: 0,
            pid: 0,
        };
        let mut inode: u64 = 0;
        let mut file_handler: u64 = 0;
        let mut offset: i64 = 0;
        let mut _write_flags: u32 = 0;
        let mut flags: i32 = 0;
        let mut _lock_owner: Option<u64> = Some(0);
        // let mut stream = in_stream.take(0);
        while let item = in_stream.next().await {
            match item {
                Some(value) => {
                    let v = value.unwrap();
                    r_data.push(v.data);
                    file_request = FileRequest {
                        uid: v.frequest.clone().uid,
                        gid: v.frequest.clone().gid,
                        pid: v.frequest.clone().pid,
                    };
                    inode = v.ino;
                    file_handler = v.fh;
                    offset = v.offset;
                    _write_flags = v.write_flag;
                    flags = v.flags;
                    _lock_owner = v.lock_owner;
                }
                None => {}
            }
        }
        let joined_data = r_data.join("");
        let data = serde_json::from_str::<&[u8]>(&joined_data).unwrap();

        let result = self
            .filesystem
            .write(
                &file_request,
                inode,
                file_handler,
                offset,
                data,
                _write_flags,
                flags,
                _lock_owner,
            )
            .await;

        match result {
            Ok(value) => Ok(Response::new(WriteReply {
                size: value.0.unwrap(),
                errcode: value.1,
            })),
            Err(_) => Err(Status::invalid_argument("write failed")),
        }
    }

    async fn lookup(
        &self,
        request: tonic::Request<LookUp>,
    ) -> Result<tonic::Response<Reply>, tonic::Status> {
        let mut request_inner = request.into_inner();
        let mut file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let mut osstring = OsString::new();
        osstring.push(request_inner.name);
        let result = self
            .filesystem
            .lookup(&file_request, request_inner.parent, &osstring.as_os_str())
            .await;

        match result {
            Ok(value) => Ok(Response::new(Reply {
                message: serde_json::to_string(&value.0.unwrap()).unwrap(),
                // message: ron::ser::to_string(&value.0.unwrap()),
                errcode: value.1,
            })),
            Err(_) => Err(Status::invalid_argument("lookup failed")),
        }
    }

    async fn create(
        &self,
        request: tonic::Request<Create>,
    ) -> Result<tonic::Response<CreateReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let mut file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let mut osstring = OsString::new();
        osstring.push(request_inner.name);
        let result = self
            .filesystem
            .create(
                &file_request,
                request_inner.parent,
                &osstring.as_os_str(),
                request_inner.mode,
                request_inner.umask,
                request_inner.flags,
            )
            .await;
        match result {
            Ok(value) => Ok(Response::new(CreateReply {
                file_attr: serde_json::to_string(&value.0.clone().unwrap().0).unwrap(),
                fh: value.0.clone().unwrap().1,
                errcode: value.1,
            })),
            Err(_) => Err(Status::invalid_argument("create failed")),
        }
    }

    async fn unlink(
        &self,
        request: tonic::Request<Unlink>,
    ) -> Result<tonic::Response<UnlinkReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let mut file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let mut osstring = OsString::new();
        osstring.push(request_inner.name);
        let result = self
            .filesystem
            .unlink(&file_request, request_inner.parent, &osstring.as_os_str())
            .await;

        match result {
            Ok(value) => Ok(Response::new(UnlinkReply { errcode: value })),
            Err(_) => Err(Status::invalid_argument("unlink failed")),
        }
    }

    async fn getattr(
        &self,
        request: tonic::Request<Getattr>,
    ) -> Result<tonic::Response<GetattrReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let mut file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let result = self
            .filesystem
            .getattr(&file_request, request_inner.ino)
            .await;

        // change fileAttr to string
        match result {
            Ok(value) => Ok(Response::new(GetattrReply {
                file_attr: serde_json::to_string(&value.0.clone().unwrap()).unwrap(),
                errcode: value.1,
            })),
            Err(_) => Err(Status::invalid_argument("getattr failed")),
        }
    }

    async fn open(
        &self,
        request: tonic::Request<Open>,
    ) -> Result<tonic::Response<OpenReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let mut file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let result = self
            .filesystem
            .open(&file_request, request_inner.ino, request_inner.flags)
            .await;

        match result {
            Ok(value) => Ok(Response::new(OpenReply {
                fh: value.0.clone().unwrap().0,
                openflag: value.0.clone().unwrap().1,
                errcode: value.1,
            })),
            Err(_) => Err(Status::invalid_argument("open failed")),
        }
    }
    async fn release(
        &self,
        request: tonic::Request<Release>,
    ) -> Result<tonic::Response<ReleaseReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let mut file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let result = self
            .filesystem
            .release(
                &file_request,
                request_inner.ino,
                request_inner.fh,
                request_inner.flags,
                request_inner.lock_owner,
                request_inner.flush,
            )
            .await;

        match result {
            Ok(value) => Ok(Response::new(ReleaseReply { errcode: value })),
            Err(_) => Err(Status::invalid_argument("release failed")),
        }
    }
    async fn setxattr(
        &self,
        request: tonic::Request<tonic::Streaming<Setxattr>>,
    ) -> Result<tonic::Response<SetxattrReply>, tonic::Status> {
        let mut in_stream = request.into_inner();

        let mut file_request: FileRequest = FileRequest {
            uid: 0,
            gid: 0,
            pid: 0,
        };
        let mut inode: u64 = 0;
        let mut name: String = "".to_string();
        let mut value_data: Vec<String> = Vec::new();
        let mut flags: i32 = 0;
        let mut position: u32 = 0;
        // let mut stream = in_stream.take(0);
        while let item = in_stream.next().await {
            match item {
                Some(value) => {
                    let v = value.unwrap();

                    value_data.push(v.value);
                    file_request = FileRequest {
                        uid: v.frequest.clone().uid,
                        gid: v.frequest.clone().gid,
                        pid: v.frequest.clone().pid,
                    };
                    inode = v.ino;
                    name = v.name;
                    flags = v.flags;
                    position = v.position;
                }
                None => {}
            }
        }
        let joined_data = value_data.join("");
        let data = serde_json::from_str::<&[u8]>(&joined_data).unwrap();

        let mut osstring = OsString::new();
        osstring.push(name);

        let result = self
            .filesystem
            .setxattr(
                &file_request,
                inode,
                &osstring.as_os_str(),
                data,
                flags,
                position,
            )
            .await;

        match result {
            Ok(value) => Ok(Response::new(SetxattrReply { errcode: value })),
            Err(_) => Err(Status::invalid_argument("setxattr failed")),
        }
    }

    async fn getxattr(
        &self,
        request: tonic::Request<Getxattr>,
    ) -> Result<tonic::Response<Self::getxattrStream>, tonic::Status> {
        let r_inner = request.into_inner();

        let request = FileRequest {
            uid: r_inner.frequest.clone().uid,
            gid: r_inner.frequest.clone().gid,
            pid: r_inner.frequest.clone().pid,
        };

        let mut osstring = OsString::new();
        osstring.push(r_inner.name);

        let result = self
            .filesystem
            .getxattr(&request, r_inner.ino, &osstring.as_os_str(), r_inner.size)
            .await;
        let mut reply = Vec::new();
        match result {
            Ok(value) => {
                if value.1 != SUCCESS {
                    reply = vec![GetxattrReply {
                        data: "".to_string(),
                        size: 0,
                        errcode: value.1,
                    }];
                } else {
                    // divide the message into appropriate size, and make a vector
                    let content = value.0.clone().unwrap().0;
                    let size = value.0.clone().unwrap().1;
                    reply = getxattr_response_iter(content, size, SUCCESS);
                }
            }
            Err(_) => {}
        };

        let mut stream = Box::pin(tokio_stream::iter(reply).throttle(Duration::from_millis(200)));
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                        // item (server response) was queued to be send to client
                    }
                    Err(_item) => {
                        // output_stream was build from rx and both are dropped
                        break;
                    }
                }
            }
            println!("\tclient disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::getxattrStream
        ))
    }

    async fn listxattr(
        &self,
        request: tonic::Request<Listxattr>,
    ) -> Result<tonic::Response<Self::listxattrStream>, tonic::Status> {
    }
    async fn access(
        &self,
        request: tonic::Request<Access>,
    ) -> Result<tonic::Response<AccessReply>, tonic::Status> {
    }
    async fn rename(
        &self,
        request: tonic::Request<Rename>,
    ) -> Result<tonic::Response<RenameReply>, tonic::Status> {
    }

    async fn setattr(
        &self,
        request: tonic::Request<Setattr>,
    ) -> Result<tonic::Response<SetattrReply>, tonic::Status> {
    }
}
