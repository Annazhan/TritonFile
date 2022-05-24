// use std::time::Duration;

// use crate::disfuser::{disfuser_server::Disfuser, Reply};
// use crate::storage::ServerFileSystem;
// use async_trait::async_trait;
// use fuser::{BackgroundSession, FileAttr, MountOption, Request};
// use tokio::sync::mpsc;
// use tokio_stream::wrappers::ReceiverStream;
// use tonic::{Response, Status};

// pub struct FuseInHeader {
//     pub len: u32,
//     pub opcode: u32,
//     pub unique: u64,
//     pub nodeid: u64,
//     pub uid: u32,
//     pub gid: u32,
//     pub pid: u32,
//     pub padding: u32,
// }

// pub struct AnyRequest<'a> {
//     header: &'a FuseInHeader,
//     data: &'a [u8],
// }

// pub struct DisfuserServer {
//     pub filesystem: Box<dyn ServerFileSystem>,
//     // pub clock: RwLock<i64>,
// }

// #[async_trait]
// impl Disfuser for DisfuserServer {
//     async fn read(
//         &self,
//         request: tonic::Request<super::Read>,
//     ) -> Result<tonic::Response<Self::readStream>, tonic::Status> {
//         // unwrap all the input Read into different values
//         // put unwrapped value into ServerFileSystem read function
//         // get read result, stringify it
//         // and then stream it and send back to client
//         let r_inner = request.into_inner();
//         let in_header = FuseInHeader {
//             len: None,
//             opcode: None,
//             unique: None,
//             nodeid: None,
//             uid: r_inner.uid,
//             gid: r_inner.gid,
//             pid: r_inner.pid,
//             padding: None,
//         };

//         let any_request = AnyRequest {
//             header: &in_header,
//             data: None,
//         };

//         let request = Request {
//             ch: None,
//             data: None,
//             request: any_request,
//         };

//         // problematic, return type is Result, current suppose res is a string
//         let result = self.filesystem.read(
//             &request,
//             r_inner.inode,
//             r_inner.fh,
//             r_inner.offset,
//             r_inner.size,
//             r_inner.flags,
//             r_inner.lock_owner,
//         );
//         let repeat;
//         match result {
//             Ok(value) => {
//                 // now value is a string, response a stream
//                 repeat = std::iter::repeat(Reply {
//                     message: value,
//                     errcod: -1,
//                 });
//             }
//             Err(err) => {
//                 repeat = std::iter::repeat(Reply {
//                     message: "",
//                     errcod: 1,
//                 });
//             }
//         };

//         let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(200)));
//         let (tx, rx) = mpsc::channel(128);
//         tokio::spawn(async move {
//             while let Some(item) = stream.next().await {
//                 match tx.send(Result::<_, Status>::Ok(item)).await {
//                     Ok(_) => {
//                         // item (server response) was queued to be send to client
//                     }
//                     Err(_item) => {
//                         // output_stream was build from rx and both are dropped
//                         break;
//                     }
//                 }
//             }
//             println!("\tclient disconnected");
//         });

//         let output_stream = ReceiverStream::new(rx);
//         Ok(Response::new(Box::pin(output_stream) as Self::readStream))
//     }

//     async fn write(
//         &self,
//         request: tonic::Request<tonic::Streaming<super::Write>>,
//     ) -> Result<tonic::Response<super::ReplyWrite>, tonic::Status> {
//         // get all data from streamed in request
//         // put them into an integrated data
//         // put unwrapped value into ServerFileSystem write function
//         // response the size data
//     }

//     async fn lookup(
//         &self,
//         request: tonic::Request<super::LookUp>,
//     ) -> Result<tonic::Response<Self::lookupStream>, tonic::Status> {
//     }

//     async fn create(
//         &self,
//         request: tonic::Request<super::Create>,
//     ) -> Result<tonic::Response<super::Bool>, tonic::Status> {
//     }

//     async fn unlink(
//         &self,
//         request: tonic::Request<super::Unlink>,
//     ) -> Result<tonic::Response<super::Bool>, tonic::Status> {
//     }

//     type readStream;

//     type lookupStream;
// }
