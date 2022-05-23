// message FuseInHeader {
//     uint32 len = 1; //  uint32
//     uint32 opcode = 2;
//     uint64 unique = 3;
//     uint64 nodeid = 4;
//     uint32 uid = 5;
//     uint32 gid = 6;
//     uint32 pid = 7;
//     uint32 padding = 8;
// }

// message FuseOutHeader {
//     uint32 len = 1; //  uint32
//     int32 error = 2;
//     uint64 unique = 3;
// }

// message AnyRequest {
//     FuseInHeader header = 1;
//     uint32 data = 2; // \[u8\]
//   }

// // Add your message and service definitions below this line
// message FRequest {
//     int64 ChannelSender = 1;
//     uint32 data = 2; // \[u8\]
//     AnyRequest request = 3;
//   }

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FRequest {
    #[prost(uint32, tag = "1")]
    pub uid: u32,
    #[prost(uint32, tag = "2")]
    pub gid: u32,
    #[prost(uint32, tag = "3")]
    pub pid: u32,
}
// message INO {
//     uint64 ino = 1;
// }

// message FH {
//     uint64 fh = 1;
// }

// message Offset {
//     int64 offset = 1;
// }

// message Size {
//     uint32 size = 1;
// }

// message Flags {
//     int32 flags = 1;
// }

// message Lock_Owner {
//     uint64 lock_owner = 1;
// }

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reply {
    /// \[u8\]
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
    /// for the Err when unwrap
    #[prost(uint64, tag = "2")]
    pub errcode: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Read {
    #[prost(message, optional, tag = "1")]
    pub frequest: ::core::option::Option<FRequest>,
    #[prost(uint64, tag = "2")]
    pub ino: u64,
    #[prost(uint64, tag = "3")]
    pub fh: u64,
    #[prost(int64, tag = "4")]
    pub offset: i64,
    #[prost(uint32, tag = "5")]
    pub size: u32,
    #[prost(int32, tag = "6")]
    pub flags: i32,
    #[prost(uint64, tag = "7")]
    pub lock_owner: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReplyWrite {
    #[prost(uint32, tag = "1")]
    pub size: u32,
    #[prost(uint64, tag = "2")]
    pub errcode: u64,
}
// message WriteFlag {
//     uint32 write_flag = 1;
// }

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    /// \[u8\]
    #[prost(uint32, tag = "1")]
    pub data: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Write {
    #[prost(message, optional, tag = "1")]
    pub frequest: ::core::option::Option<FRequest>,
    #[prost(uint64, tag = "2")]
    pub ino: u64,
    #[prost(uint64, tag = "3")]
    pub fh: u64,
    #[prost(int64, tag = "4")]
    pub offset: i64,
    /// \[u8\]
    #[prost(string, tag = "5")]
    pub data: ::prost::alloc::string::String,
    #[prost(uint32, tag = "6")]
    pub write_flag: u32,
    #[prost(int32, tag = "7")]
    pub flags: i32,
    #[prost(uint64, tag = "8")]
    pub lock_owner: u64,
}
// message Parent {
//     uint64 parent = 1;
// }

// message OsStr {
//     Slice inner = 1;
// }

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LookUp {
    #[prost(message, optional, tag = "1")]
    pub frequest: ::core::option::Option<FRequest>,
    #[prost(uint64, tag = "2")]
    pub parent: u64,
    /// OsStr name = 3;
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
}
// message Slice {
//     uint8 inner = 1; // \[u8\]
// }

// message Mode {
//     uint32 mode = 1;
// }

// message Umask {
//     uint32 mask = 1;
// }

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Create {
    #[prost(message, optional, tag = "1")]
    pub frequest: ::core::option::Option<FRequest>,
    #[prost(uint64, tag = "2")]
    pub parent: u64,
    /// OsStr name = 3;
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint32, tag = "4")]
    pub mode: u32,
    #[prost(uint32, tag = "5")]
    pub umask: u32,
    #[prost(int32, tag = "6")]
    pub flags: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bool {
    #[prost(bool, tag = "1")]
    pub bool: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Unlink {
    #[prost(message, optional, tag = "1")]
    pub frequest: ::core::option::Option<FRequest>,
    #[prost(uint64, tag = "2")]
    pub parent: u64,
    /// OsStr name = 3;
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateRply {
    #[prost(string, tag = "1")]
    pub file_attr: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub code: u64,
}
#[doc = r" Generated client implementations."]
pub mod disfuser_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct DisfuserClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl DisfuserClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> DisfuserClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> DisfuserClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            DisfuserClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn read(
            &mut self,
            request: impl tonic::IntoRequest<super::Read>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::Reply>>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/read");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn write(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::Write>,
        ) -> Result<tonic::Response<super::ReplyWrite>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/write");
            self.inner
                .client_streaming(request.into_streaming_request(), path, codec)
                .await
        }
        pub async fn lookup(
            &mut self,
            request: impl tonic::IntoRequest<super::LookUp>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::Reply>>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/lookup");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn create(
            &mut self,
            request: impl tonic::IntoRequest<super::Create>,
        ) -> Result<tonic::Response<super::CreateRply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/create");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn unlink(
            &mut self,
            request: impl tonic::IntoRequest<super::Unlink>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/unlink");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod disfuser_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with DisfuserServer."]
    #[async_trait]
    pub trait Disfuser: Send + Sync + 'static {
        #[doc = "Server streaming response type for the read method."]
        type readStream: futures_core::Stream<Item = Result<super::Reply, tonic::Status>>
            + Send
            + 'static;
        async fn read(
            &self,
            request: tonic::Request<super::Read>,
        ) -> Result<tonic::Response<Self::readStream>, tonic::Status>;
        async fn write(
            &self,
            request: tonic::Request<tonic::Streaming<super::Write>>,
        ) -> Result<tonic::Response<super::ReplyWrite>, tonic::Status>;
        #[doc = "Server streaming response type for the lookup method."]
        type lookupStream: futures_core::Stream<Item = Result<super::Reply, tonic::Status>>
            + Send
            + 'static;
        async fn lookup(
            &self,
            request: tonic::Request<super::LookUp>,
        ) -> Result<tonic::Response<Self::lookupStream>, tonic::Status>;
        async fn create(
            &self,
            request: tonic::Request<super::Create>,
        ) -> Result<tonic::Response<super::CreateRply>, tonic::Status>;
        async fn unlink(
            &self,
            request: tonic::Request<super::Unlink>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct DisfuserServer<T: Disfuser> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Disfuser> DisfuserServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for DisfuserServer<T>
    where
        T: Disfuser,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/disfuser.disfuser/read" => {
                    #[allow(non_camel_case_types)]
                    struct readSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::ServerStreamingService<super::Read> for readSvc<T> {
                        type Response = super::Reply;
                        type ResponseStream = T::readStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Read>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).read(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = readSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/disfuser.disfuser/write" => {
                    #[allow(non_camel_case_types)]
                    struct writeSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::ClientStreamingService<super::Write> for writeSvc<T> {
                        type Response = super::ReplyWrite;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::Write>>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).write(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = writeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.client_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/disfuser.disfuser/lookup" => {
                    #[allow(non_camel_case_types)]
                    struct lookupSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::ServerStreamingService<super::LookUp> for lookupSvc<T> {
                        type Response = super::Reply;
                        type ResponseStream = T::lookupStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::LookUp>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).lookup(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = lookupSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/disfuser.disfuser/create" => {
                    #[allow(non_camel_case_types)]
                    struct createSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Create> for createSvc<T> {
                        type Response = super::CreateRply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Create>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = createSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/disfuser.disfuser/unlink" => {
                    #[allow(non_camel_case_types)]
                    struct unlinkSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Unlink> for unlinkSvc<T> {
                        type Response = super::Bool;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Unlink>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).unlink(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = unlinkSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Disfuser> Clone for DisfuserServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Disfuser> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Disfuser> tonic::transport::NamedService for DisfuserServer<T> {
        const NAME: &'static str = "disfuser.disfuser";
    }
}
