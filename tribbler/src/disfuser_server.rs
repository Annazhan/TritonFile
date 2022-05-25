use crate::disfuser::disfuser_server::Disfuser;
use crate::disfuser::{
    Create, CreateReply, LookUp, Read, Reply, Unlink, UnlinkReply, Write, WriteReply,
};
use crate::storage::FileRequest;
use crate::storage::Storage;
use async_trait::async_trait;
use fuser::{BackgroundSession, FileAttr, MountOption, Request};
use std::cmp::min;
use std::ffi::OsStr;
use std::pin::Pin;
use sys::os_str::Slice;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tonic::{Response, Status};

type readStream = Pin<Box<dyn Stream<Item = Result<Reply, Status>> + Send>>;
// type readStream = Pin<Box<dyn Stream<Item = Result<Read, Status>> + Send>>;
// type lookupStream = Pin<Box<dyn Stream<Item = Result<LookUp, Status>> + Send>>;

pub struct DisfuserServer {
    pub filesystem: Box<dyn Storage>,
    // pub clock: RwLock<i64>,
}

fn reply_response_iter(msg: String, errcode: i32) -> Vec<Reply> {
    let slice_size = 128;
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

#[async_trait]
impl Disfuser for DisfuserServer {
    type readStream = readStream;

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
            uid: r_inner.frequest.unwrap().uid,
            gid: r_inner.frequest.unwrap().gid,
            pid: r_inner.frequest.unwrap().pid,
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
                Some(r_inner.lock_owner),
            )
            .await;

        let reply;
        match result {
            Ok(value) => {
                if value.1 > 0 {
                    reply = vec![Reply {
                        message: "".to_string(),
                        errcode: value.1,
                    }];
                } else {
                    // divide the message into appropriate size, and make a vector
                    let content = value.0.unwrap();
                    reply = reply_response_iter(content, -1);
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
        let mut file_request: FileRequest;
        let mut inode: u64;
        let mut file_handler: u64;
        let mut offset: i64;
        let mut _write_flags: u32;
        let mut flags: i32;
        let mut _lock_owner: Option<u64>;
        // let mut stream = in_stream.take(0);
        while let item = in_stream.next().await {
            match item {
                Some(value) => {
                    let v = value.unwrap();
                    r_data.push(v.data);
                    file_request = FileRequest {
                        uid: v.frequest.unwrap().uid,
                        gid: v.frequest.unwrap().gid,
                        pid: v.frequest.unwrap().pid,
                    };
                    inode = v.ino;
                    file_handler = v.fh;
                    offset = v.offset;
                    _write_flags = v.write_flag;
                    flags = v.flags;
                    _lock_owner = Some(v.lock_owner);
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
        // let request_inner = request.into_inner();
        // let mut file_request = FileRequest {
        //     uid: request_inner.frequest.unwrap().uid,
        //     gid: request_inner.frequest.unwrap().gid,
        //     pid: request_inner.frequest.unwrap().pid,
        // };
        // let mut name = serde_json::from_str::<[FileAttr]>(&request_inner.name).unwrap();
        // name = OsStr {
        //     inner: Slice { inner: *name },
        // };
        // let result = self
        //     .filesystem
        //     .lookup(&file_request, request_inner.parent, &name)
        //     .await;

        // match result {
        //     Ok(value) => Ok(Response::new(Reply {
        //         message: serde_json::to_string(&value.0.unwrap()),
        //         errcode: value.1,
        //     })),
        //     Err(_) => Err(Status::invalid_argument("lookup failed")),
        // }
    }

    async fn create(
        &self,
        request: tonic::Request<Create>,
    ) -> Result<tonic::Response<CreateReply>, tonic::Status> {
        let request_inner = request.into_inner();
    }

    async fn unlink(
        &self,
        request: tonic::Request<Unlink>,
    ) -> Result<tonic::Response<UnlinkReply>, tonic::Status> {
        let request_inner = request.into_inner();
    }
}
