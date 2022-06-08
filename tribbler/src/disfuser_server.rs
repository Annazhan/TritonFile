use crate::disfuser::disfuser_server::{self, Disfuser};
use crate::disfuser::{
    self, Access, AccessReply, Create, CreateReply, GetAllNodes, GetAllNodesReply, Getattr,
    GetattrReply, Getxattr, GetxattrReply, Init, InitReply, Listxattr, ListxattrReply, LookUp,
    MkDir, MkDirReply, Open, OpenDir, OpenDirReply, OpenReply, Read, ReadDir, ReadDirReply,
    Release, ReleaseDir, ReleaseDirReply, ReleaseReply, Rename, RenameReply, Reply, Setattr,
    SetattrReply, Setxattr, SetxattrReply, Unlink, UnlinkReply, Write, WriteAllNodes,
    WriteAllNodesReply, WriteReply,
};
use crate::error::SUCCESS;
use crate::simple::InodeAttributes;
use crate::storage::{ContentList, DataList, FileRequest, InodeList, Storage};
use async_trait::async_trait;
use fuser::{BackgroundSession, FileAttr, MountOption, Request, TimeOrNow};
use log::info;
use std::cmp::min;
use std::ffi::OsString;
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
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
use crate::disfuser::{Clock, Key, KeyValue, StringList, Value};
use crate::storage;
#[allow(dead_code)]
pub struct DisfuserServer {
    pub filesystem: Box<dyn Storage>,
    // pub clock: RwLock<i64>,
}

#[allow(dead_code)]
fn reply_response_iter(msg: String, errcode: i32) -> Vec<Reply> {
    let data_len = msg.len();

    let mut n = data_len / slice_size;
    if data_len % slice_size != 0 {
        n += 1;
    }

    let mut vec: Vec<Reply> = Vec::new();
    let mut start;
    let mut end;

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
#[allow(dead_code)]
fn getxattr_response_iter(msg: String, size: u32, errcode: i32) -> Vec<GetxattrReply> {
    let data_len = msg.len();
    let mut n = data_len / slice_size;
    if data_len % slice_size != 0 {
        n += 1;
    }

    let mut vec: Vec<GetxattrReply> = Vec::new();
    let mut start;
    let mut end;

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
#[allow(dead_code)]
fn listxattr_response_iter(msg: String, size: u32, errcode: i32) -> Vec<ListxattrReply> {
    let data_len = msg.len();
    let mut n = data_len / slice_size;
    if data_len % slice_size != 0 {
        n += 1;
    }

    let mut vec: Vec<ListxattrReply> = Vec::new();
    let mut start;
    let mut end;

    for i in 0..n {
        start = i * slice_size;
        end = min(start + slice_size, data_len);
        let element = ListxattrReply {
            data: msg.clone()[start..end].to_string(),
            size: size,
            errcode: errcode,
        };
        vec.push(element.clone());
    }
    return vec;
}
#[allow(dead_code)]
fn system_time_from_time(secs: i64, nsecs: u32) -> SystemTime {
    if secs >= 0 {
        UNIX_EPOCH + Duration::new(secs as u64, nsecs)
    } else {
        UNIX_EPOCH - Duration::new((-secs) as u64, nsecs)
    }
}
#[allow(dead_code)]
impl DisfuserServer {
    pub fn new(storage: Box<dyn storage::Storage>) -> DisfuserServer {
        DisfuserServer {
            filesystem: storage,
        }
    }
}

#[async_trait]
#[allow(dead_code)]
impl Disfuser for DisfuserServer {
    type readStream = readStream;
    type getxattrStream = getxattrStream;
    type listxattrStream = listxattrStream;

    async fn init(
        &self,
        request: tonic::Request<Init>,
    ) -> Result<tonic::Response<InitReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };

        let result = self.filesystem.init(&file_request).await;

        // change fileAttr to string
        match result {
            Ok(value) => Ok(Response::new(InitReply { errcode: value })),
            Err(_) => Err(Status::invalid_argument("rename failed")),
        }
    }

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
            println!("\t read client disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::readStream))
    }

    // async fn write(
    //     &self,
    //     request: tonic::Request<tonic::Streaming<Write>>,
    // ) -> Result<tonic::Response<WriteReply>, tonic::Status> {
    //     let mut in_stream = request.into_inner();

    //     let mut r_data: Vec<String> = Vec::new();
    //     let mut file_request: FileRequest = FileRequest {
    //         uid: 0,
    //         gid: 0,
    //         pid: 0,
    //     };
    //     let mut inode: u64 = 0;
    //     let mut file_handler: u64 = 0;
    //     let mut offset: i64 = 0;
    //     let mut _write_flags: u32 = 0;
    //     let mut flags: i32 = 0;
    //     let mut _lock_owner: Option<u64> = Some(0);
    //     // let mut stream = in_stream.take(0);
    //     while let item = in_stream.next().await {
    //         match item {
    //             Some(value) => {
    //                 let v = value.unwrap();
    //                 r_data.push(v.data);
    //                 file_request = FileRequest {
    //                     uid: v.frequest.clone().uid,
    //                     gid: v.frequest.clone().gid,
    //                     pid: v.frequest.clone().pid,
    //                 };
    //                 inode = v.ino;
    //                 file_handler = v.fh;
    //                 offset = v.offset;
    //                 _write_flags = v.write_flag;
    //                 flags = v.flags;
    //                 _lock_owner = v.lock_owner;
    //             }
    //             None => {}
    //         }
    //     }
    //     let joined_data = r_data.join("");
    //     let data = serde_json::from_str::<&[u8]>(&joined_data).unwrap();

    //     let result = self
    //         .filesystem
    //         .write(
    //             &file_request,
    //             inode,
    //             file_handler,
    //             offset,
    //             data,
    //             _write_flags,
    //             flags,
    //             _lock_owner,
    //         )
    //         .await;

    //     match result {
    //         Ok((size, errcode)) => {
    //             if errcode != SUCCESS {
    //                 return Ok(Response::new(WriteReply {
    //                     size: 0,
    //                     errcode: errcode,
    //                 }));
    //             } else {
    //                 return Ok(Response::new(WriteReply {
    //                     size: size.unwrap(),
    //                     errcode: errcode,
    //                 }));
    //             };
    //         }
    //         Err(_) => Err(Status::invalid_argument("write failed")),
    //     }
    // }

    async fn write(
        &self,
        request: tonic::Request<Write>,
    ) -> Result<tonic::Response<WriteReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        info!("disfuser_write write before");
        info!("{}", request_inner.data);
        let result = self
            .filesystem
            .write(
                &file_request,
                request_inner.ino,
                request_inner.fh,
                request_inner.offset,
                // &(serde_json::from_str::<[u8]>(&).unwrap()),
                request_inner.data.as_bytes(),
                request_inner.write_flag,
                request_inner.flags,
                request_inner.lock_owner,
            )
            .await;
        info!("disfuser_write write after");
        match result {
            Ok((value, errcode)) => {
                if errcode != SUCCESS {
                    Ok(Response::new(WriteReply { size: 0, errcode }))
                } else {
                    Ok(Response::new(WriteReply {
                        size: value.unwrap(),
                        errcode,
                    }))
                }
            }
            Err(_) => Err(Status::invalid_argument("write failed")),
        }
    }

    async fn lookup(
        &self,
        request: tonic::Request<LookUp>,
    ) -> Result<tonic::Response<Reply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
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
            Ok((file_attr, errcode)) => {
                if errcode != SUCCESS {
                    Ok(Response::new(Reply {
                        message: "".to_string(),
                        errcode: errcode,
                    }))
                } else {
                    Ok(Response::new(Reply {
                        message: serde_json::to_string(&file_attr.unwrap()).unwrap(),
                        errcode: errcode,
                    }))
                }
            }
            Err(_) => Err(Status::invalid_argument("lookup failed")),
        }
    }

    async fn create(
        &self,
        request: tonic::Request<Create>,
    ) -> Result<tonic::Response<CreateReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
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
            Ok((res_op, errcode)) => {
                if errcode != SUCCESS {
                    return Ok(Response::new(CreateReply {
                        file_attr: "".to_string(),
                        fh: 0,
                        errcode: errcode,
                    }));
                } else {
                    return Ok(Response::new(CreateReply {
                        file_attr: serde_json::to_string(&res_op.clone().unwrap().0).unwrap(),
                        fh: res_op.clone().unwrap().1,
                        errcode: errcode,
                    }));
                };
            }
            Err(_) => Err(Status::invalid_argument("create failed")),
        }
    }

    async fn unlink(
        &self,
        request: tonic::Request<Unlink>,
    ) -> Result<tonic::Response<UnlinkReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
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
        let file_request = FileRequest {
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
            Ok((file_attr, errcode)) => {
                if errcode != SUCCESS {
                    return Ok(Response::new(GetattrReply {
                        file_attr: "".to_string(),
                        errcode: errcode,
                    }));
                } else {
                    return Ok(Response::new(GetattrReply {
                        file_attr: serde_json::to_string(&file_attr.unwrap()).unwrap(),
                        errcode: errcode,
                    }));
                }
            }
            Err(_) => Err(Status::invalid_argument("getattr failed")),
        }
    }

    async fn open(
        &self,
        request: tonic::Request<Open>,
    ) -> Result<tonic::Response<OpenReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let result = self
            .filesystem
            .open(&file_request, request_inner.ino, request_inner.flags)
            .await;

        match result {
            Ok(value) => {
                if value.1 != SUCCESS {
                    return Ok(Response::new(OpenReply {
                        fh: 0,
                        openflag: 0,
                        errcode: value.1,
                    }));
                } else {
                    return Ok(Response::new(OpenReply {
                        fh: value.0.clone().unwrap().0,
                        openflag: value.0.clone().unwrap().1,
                        errcode: value.1,
                    }));
                }
            }
            Err(_) => Err(Status::invalid_argument("open failed")),
        }
    }
    async fn release(
        &self,
        request: tonic::Request<Release>,
    ) -> Result<tonic::Response<ReleaseReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
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
        let data = joined_data.as_bytes();

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
        info!("rpc server getxattr, filesystem result: {:?}", result);
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
            info!("\tgetxattr client disconnected");
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
        let r_inner = request.into_inner();

        let request = FileRequest {
            uid: r_inner.frequest.clone().uid,
            gid: r_inner.frequest.clone().gid,
            pid: r_inner.frequest.clone().pid,
        };

        let result = self
            .filesystem
            .listxattr(&request, r_inner.ino, r_inner.size)
            .await;
        let mut reply = Vec::new();
        match result {
            Ok(value) => {
                if value.1 != SUCCESS {
                    reply = vec![ListxattrReply {
                        data: "".to_string(),
                        size: 0,
                        errcode: value.1,
                    }];
                } else {
                    // divide the message into appropriate size, and make a vector
                    let content = value.0.clone().unwrap().0;
                    let size = value.0.clone().unwrap().1;
                    reply = listxattr_response_iter(content, size, SUCCESS);
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
            info!("\t listxattr client disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::listxattrStream
        ))
    }
    async fn access(
        &self,
        request: tonic::Request<Access>,
    ) -> Result<tonic::Response<AccessReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let result = self
            .filesystem
            .access(&file_request, request_inner.ino, request_inner.mask)
            .await;

        // change fileAttr to string
        match result {
            Ok(value) => Ok(Response::new(AccessReply { errcode: value })),
            Err(_) => Err(Status::invalid_argument("access failed")),
        }
    }

    async fn rename(
        &self,
        request: tonic::Request<Rename>,
    ) -> Result<tonic::Response<RenameReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };

        let mut old_name = OsString::new();
        old_name.push(request_inner.name);

        let mut new_name = OsString::new();
        new_name.push(request_inner.newname);

        let result = self
            .filesystem
            .rename(
                &file_request,
                request_inner.parent,
                &old_name.as_os_str(),
                request_inner.newparent,
                &new_name.as_os_str(),
                request_inner.flags,
            )
            .await;

        // change fileAttr to string
        match result {
            Ok(value) => Ok(Response::new(RenameReply { errcode: value })),
            Err(_) => Err(Status::invalid_argument("rename failed")),
        }
    }

    async fn setattr(
        &self,
        request: tonic::Request<Setattr>,
    ) -> Result<tonic::Response<SetattrReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };

        let atime = match request_inner.atime_secs {
            Some(atime_secs) => Some(TimeOrNow::SpecificTime(system_time_from_time(
                atime_secs,
                request_inner.atime_nsecs.unwrap(),
            ))),
            None => None,
        };

        let mtime = match request_inner.mtime_secs {
            Some(mtime_secs) => Some(TimeOrNow::SpecificTime(system_time_from_time(
                mtime_secs,
                request_inner.mtime_nsecs.unwrap(),
            ))),
            None => None,
        };

        let empty_time = Some(system_time_from_time(0, 0));

        let result = self
            .filesystem
            .setattr(
                &file_request,
                request_inner.ino,
                request_inner.mode,
                request_inner.uid,
                request_inner.gid,
                request_inner.size,
                atime,
                mtime,
                empty_time,
                request_inner.fh,
                empty_time,
                empty_time,
                empty_time,
                request_inner.flags,
            )
            .await;

        // change fileAttr to string
        match result {
            Ok(value) => {
                if value.1 != SUCCESS {
                    return Ok(Response::new(SetattrReply {
                        file_attr: "".to_string(),
                        errcode: value.1,
                    }));
                } else {
                    return Ok(Response::new(SetattrReply {
                        file_attr: serde_json::to_string(&value.0.clone().unwrap()).unwrap(),
                        errcode: value.1,
                    }));
                }
            }
            Err(_) => Err(Status::invalid_argument("setattr failed")),
        }
    }

    async fn readdir(
        &self,
        request: tonic::Request<ReadDir>,
    ) -> Result<tonic::Response<ReadDirReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let result = self
            .filesystem
            .readdir(
                &file_request,
                request_inner.ino,
                request_inner.fh,
                request_inner.offset,
            )
            .await;

        // change fileAttr to string
        match result {
            Ok((value, errcode)) => {
                if errcode != SUCCESS {
                    Ok(Response::new(ReadDirReply {
                        ino: None,
                        offset: None,
                        file_type: None,
                        name: None,
                        errcode,
                    }))
                } else {
                    match value {
                        Some(result) => Ok(Response::new(ReadDirReply {
                            ino: Some(result.0),
                            offset: Some(result.1),
                            file_type: Some(serde_json::to_string(&result.2).unwrap()),
                            name: Some(std::str::from_utf8(&result.3 .0).unwrap().to_string()),
                            errcode,
                        })),
                        None => Ok(Response::new(ReadDirReply {
                            ino: None,
                            offset: None,
                            file_type: None,
                            name: None,
                            errcode,
                        })),
                    }
                }
            }
            Err(_) => Err(Status::invalid_argument("readdir failed")),
        }
    }

    async fn get_all_nodes(
        &self,
        request: tonic::Request<GetAllNodes>,
    ) -> Result<tonic::Response<GetAllNodesReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let result = self
            .filesystem
            .get_all_nodes(request_inner.for_addr as usize, request_inner.len as usize)
            .await;

        match result {
            Ok(value) => match value {
                Some(v) => {
                    let inode_list = v.0;
                    let inode_vec = inode_list.0;
                    let mut inode_s = Vec::new();
                    for i in inode_vec.iter() {
                        inode_s.push(serde_json::to_string(&i).unwrap())
                    }

                    let content_list = v.1;
                    let contect_vec = content_list.0;
                    let mut contect_s = Vec::new();
                    for i in contect_vec.iter() {
                        let data = &i.0;
                        contect_s.push(std::str::from_utf8(data).unwrap().to_string());
                    }

                    Ok(Response::new(GetAllNodesReply {
                        file_attr: inode_s,
                        data_s: contect_s,
                        errcode: SUCCESS,
                    }))
                }
                None => {
                    let empty_vec: Vec<String> = Vec::new();
                    Ok(Response::new(GetAllNodesReply {
                        file_attr: empty_vec.clone(),
                        data_s: empty_vec,
                        errcode: 1,
                    }))
                }
            },

            Err(_) => Err(Status::invalid_argument("get_all_nodes failed")),
        }
    }

    async fn write_all_nodes(
        &self,
        request: tonic::Request<WriteAllNodes>,
    ) -> Result<tonic::Response<WriteAllNodesReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let mut inode_vec: Vec<InodeAttributes> = Vec::new();
        for file_attr in request_inner.file_attr {
            let inode_attr = serde_json::from_str::<InodeAttributes>(&file_attr).unwrap();
            inode_vec.push(inode_attr);
        }
        let inode_list = InodeList(inode_vec);

        let mut data_vec: Vec<DataList> = Vec::new();
        for data in request_inner.data_s {
            let data_list = DataList((*data.as_bytes()).to_vec());
            data_vec.push(data_list);
        }
        let content_list = ContentList(data_vec);

        let result = self
            .filesystem
            .write_all_nodes(inode_list, content_list)
            .await;

        match result {
            Ok(_value) => Ok(Response::new(WriteAllNodesReply { errcode: SUCCESS })),
            Err(_) => Err(Status::invalid_argument("write_all_nodes failed")),
        }
    }

    async fn opendir(
        &self,
        request: tonic::Request<OpenDir>,
    ) -> Result<tonic::Response<OpenDirReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let result = self
            .filesystem
            .opendir(&file_request, request_inner.ino, request_inner.flags)
            .await;

        // change fileAttr to string
        match result {
            Ok((value, errcode)) => {
                if errcode != SUCCESS {
                    Ok(Response::new(OpenDirReply {
                        fh: 0,
                        flags: 0,
                        errcode,
                    }))
                } else {
                    let v = value.unwrap();
                    Ok(Response::new(OpenDirReply {
                        fh: v.0,
                        flags: v.1,
                        errcode,
                    }))
                }
            }
            Err(_) => Err(Status::invalid_argument("getattr failed")),
        }
    }

    async fn mkdir(
        &self,
        request: tonic::Request<MkDir>,
    ) -> Result<tonic::Response<MkDirReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };

        let mut name = OsString::new();
        name.push(request_inner.name);

        let result = self
            .filesystem
            .mkdir(
                &file_request,
                request_inner.parent,
                &name.as_os_str(),
                request_inner.mode,
                request_inner.umask,
            )
            .await;

        // change fileAttr to string
        match result {
            Ok((value, errcode)) => {
                if errcode != SUCCESS {
                    Ok(Response::new(MkDirReply {
                        fileattr: "".to_string(),
                        errcode,
                    }))
                } else {
                    let v = value.unwrap();
                    Ok(Response::new(MkDirReply {
                        fileattr: serde_json::to_string(&v).unwrap(),
                        errcode,
                    }))
                }
            }
            Err(_) => Err(Status::invalid_argument("getattr failed")),
        }
    }

    async fn releasedir(
        &self,
        request: tonic::Request<ReleaseDir>,
    ) -> Result<tonic::Response<ReleaseDirReply>, tonic::Status> {
        let request_inner = request.into_inner();
        let file_request = FileRequest {
            uid: request_inner.frequest.clone().uid,
            gid: request_inner.frequest.clone().gid,
            pid: request_inner.frequest.clone().pid,
        };
        let result = self
            .filesystem
            .releasedir(
                &file_request,
                request_inner.inode,
                request_inner.fh,
                request_inner.flags,
            )
            .await;

        // change fileAttr to string
        match result {
            Ok(value) => Ok(Response::new(ReleaseDirReply { errcode: value })),
            Err(_) => Err(Status::invalid_argument("getattr failed")),
        }
    }

    ////////////////////////////////////////////////////////////////////
    /// below is the storage part
    async fn get(
        &self,
        request: tonic::Request<Key>,
    ) -> Result<tonic::Response<Value>, tonic::Status> {
        let key = request.into_inner().key;
        let ret = self.filesystem.get(key.as_str()).await;
        info!("Get request, key: {:}, ret: {:?}", &key, &ret);
        match ret {
            Ok(val) => match val {
                Some(value) => Ok(Response::new(disfuser::Value { value })),
                None => Err(Status::not_found("Value not found")),
            },
            Err(e) => Err(Status::unknown(e.to_string())),
        }
    }

    async fn set(
        &self,
        request: tonic::Request<disfuser::KeyValue>,
    ) -> Result<tonic::Response<disfuser::Bool>, tonic::Status> {
        let req = request.into_inner();
        let key = req.key;
        let value = req.value;
        let ret = self
            .filesystem
            .set(&storage::KeyValue {
                key: key.clone(),
                value: value.clone(),
            })
            .await;
        info!(
            "Set request, key: {:}, val: {:}, ret: {:?}",
            key, value, &ret
        );
        match ret {
            Ok(value) => Ok(Response::new(disfuser::Bool { value })),
            Err(e) => Err(Status::unknown(e.to_string())),
        }
    }

    async fn keys(
        &self,
        request: tonic::Request<disfuser::Pattern>,
    ) -> Result<tonic::Response<disfuser::StringList>, tonic::Status> {
        let req = request.into_inner();
        let prefix = req.prefix;
        let suffix = req.suffix;
        let ret = self
            .filesystem
            .keys(&storage::Pattern { prefix, suffix })
            .await;
        info!("Keys request, ret: {:?}", &ret);
        match ret {
            Ok(list) => Ok(Response::new(disfuser::StringList { list: list.0 })),
            Err(e) => Err(Status::unknown(e.to_string())),
        }
    }

    async fn list_get(
        &self,
        request: tonic::Request<disfuser::Key>,
    ) -> Result<tonic::Response<disfuser::StringList>, tonic::Status> {
        let key = request.into_inner().key;
        let ret = self.filesystem.list_get(key.as_str()).await;
        info!("List_get request, key: {:}, ret: {:?}", key, &ret);
        match ret {
            Ok(list) => Ok(Response::new(disfuser::StringList { list: list.0 })),
            Err(e) => Err(Status::unknown(e.to_string())),
        }
    }

    async fn list_keys(
        &self,
        request: tonic::Request<disfuser::Pattern>,
    ) -> Result<tonic::Response<disfuser::StringList>, tonic::Status> {
        let req = request.into_inner();
        let prefix = req.prefix;
        let suffix = req.suffix;
        let ret = self
            .filesystem
            .list_keys(&storage::Pattern { prefix, suffix })
            .await;
        info!("List_keys request, ret: {:?}", &ret);
        match ret {
            Ok(list) => Ok(Response::new(disfuser::StringList { list: list.0 })),
            Err(e) => Err(Status::unknown(e.to_string())),
        }
    }

    async fn list_append(
        &self,
        request: tonic::Request<disfuser::KeyValue>,
    ) -> Result<tonic::Response<disfuser::Bool>, tonic::Status> {
        let req = request.into_inner();
        let key = req.key;
        let value = req.value;
        let ret = self
            .filesystem
            .list_append(&storage::KeyValue {
                key: key.clone(),
                value: value.clone(),
            })
            .await;
        let result = self.filesystem.list_get(&key).await;
        info!(
            "List_append request, key: {:}, val: {:}, ret: {:?}, result: {:?}",
            &key, &value, &ret, &result
        );
        match ret {
            Ok(value) => Ok(Response::new(disfuser::Bool { value })),
            Err(e) => Err(Status::unknown(e.to_string())),
        }
    }

    async fn list_remove(
        &self,
        request: tonic::Request<disfuser::KeyValue>,
    ) -> Result<tonic::Response<disfuser::ListRemoveResponse>, tonic::Status> {
        let req = request.into_inner();
        let key = req.key;
        let value = req.value;
        let ret = self
            .filesystem
            .list_remove(&storage::KeyValue {
                key: key.clone(),
                value,
            })
            .await;
        let result = self.filesystem.list_get(&key).await;
        info!(
            "List_remove request, key: {:}, ret: {:?}, result: {:?}",
            key, &ret, result
        );
        match ret {
            Ok(removed) => Ok(Response::new(disfuser::ListRemoveResponse { removed })),
            Err(e) => Err(Status::unknown(e.to_string())),
        }
    }

    async fn clock(
        &self,
        request: tonic::Request<disfuser::Clock>,
    ) -> Result<tonic::Response<disfuser::Clock>, tonic::Status> {
        let at_least = request.into_inner().timestamp;
        let ret = self.filesystem.clock(at_least).await;
        // info!("Clock request, at_least: {:}, ret: {:?}", at_least, &ret);
        match ret {
            Ok(timestamp) => Ok(Response::new(disfuser::Clock { timestamp })),
            Err(e) => Err(Status::unknown(e.to_string())),
        }
    }
}
