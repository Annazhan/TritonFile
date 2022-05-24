#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FuseInHeader {
    ///  uint32
    #[prost(uint32, tag = "1")]
    pub len: u32,
    #[prost(uint32, tag = "2")]
    pub opcode: u32,
    #[prost(uint64, tag = "3")]
    pub unique: u64,
    #[prost(uint64, tag = "4")]
    pub nodeid: u64,
    #[prost(uint32, tag = "5")]
    pub uid: u32,
    #[prost(uint32, tag = "6")]
    pub gid: u32,
    #[prost(uint32, tag = "7")]
    pub pid: u32,
    #[prost(uint32, tag = "8")]
    pub padding: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FuseOutHeader {
    ///  uint32
    #[prost(uint32, tag = "1")]
    pub len: u32,
    #[prost(int32, tag = "2")]
    pub error: i32,
    #[prost(uint64, tag = "3")]
    pub unique: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnyRequest {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<FuseInHeader>,
    /// \[u8\]
    #[prost(uint32, tag = "2")]
    pub data: u32,
}
/// Add your message and service definitions below this line
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FRequest {
    #[prost(int64, tag = "1")]
    pub channel_sender: i64,
    /// \[u8\]
    #[prost(uint32, tag = "2")]
    pub data: u32,
    #[prost(message, optional, tag = "3")]
    pub request: ::core::option::Option<AnyRequest>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ino {
    #[prost(uint64, tag = "1")]
    pub ino: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Fh {
    #[prost(uint64, tag = "1")]
    pub fh: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Offset {
    #[prost(int64, tag = "1")]
    pub offset: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Size {
    #[prost(uint32, tag = "1")]
    pub size: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Flags {
    #[prost(int32, tag = "1")]
    pub flags: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockOwner {
    #[prost(uint64, tag = "1")]
    pub lock_owner: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reply {
    /// \[u8\]
    #[prost(uint32, tag = "1")]
    pub message: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Read {
    #[prost(message, optional, tag = "1")]
    pub frequest: ::core::option::Option<FRequest>,
    #[prost(message, optional, tag = "2")]
    pub ino: ::core::option::Option<Ino>,
    #[prost(message, optional, tag = "3")]
    pub fh: ::core::option::Option<Fh>,
    #[prost(message, optional, tag = "4")]
    pub offset: ::core::option::Option<Offset>,
    #[prost(message, optional, tag = "5")]
    pub size: ::core::option::Option<Size>,
    #[prost(message, optional, tag = "6")]
    pub flags: ::core::option::Option<Flags>,
    #[prost(message, optional, tag = "7")]
    pub lock_owner: ::core::option::Option<LockOwner>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReplyWrite {
    #[prost(uint32, tag = "1")]
    pub size: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteFlag {
    #[prost(uint32, tag = "1")]
    pub write_flag: u32,
}
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
    #[prost(message, optional, tag = "2")]
    pub ino: ::core::option::Option<Ino>,
    #[prost(message, optional, tag = "3")]
    pub fh: ::core::option::Option<Fh>,
    #[prost(message, optional, tag = "4")]
    pub offset: ::core::option::Option<Offset>,
    #[prost(message, optional, tag = "5")]
    pub data: ::core::option::Option<Data>,
    #[prost(message, optional, tag = "6")]
    pub write_flag: ::core::option::Option<WriteFlag>,
    #[prost(message, optional, tag = "7")]
    pub flags: ::core::option::Option<Flags>,
    #[prost(message, optional, tag = "8")]
    pub lock_owner: ::core::option::Option<LockOwner>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Parent {
    #[prost(uint64, tag = "1")]
    pub parent: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OsStr {
    #[prost(message, optional, tag = "1")]
    pub inner: ::core::option::Option<Slice>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LookUp {
    #[prost(message, optional, tag = "1")]
    pub frequest: ::core::option::Option<FRequest>,
    #[prost(message, optional, tag = "2")]
    pub parent: ::core::option::Option<Parent>,
    #[prost(message, optional, tag = "3")]
    pub name: ::core::option::Option<OsStr>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Slice {
    /// \[u8\]
    #[prost(uint32, tag = "1")]
    pub inner: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Mode {
    #[prost(uint32, tag = "1")]
    pub mode: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Umask {
    #[prost(uint32, tag = "1")]
    pub mask: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Create {
    #[prost(message, optional, tag = "1")]
    pub frequest: ::core::option::Option<FRequest>,
    #[prost(message, optional, tag = "2")]
    pub parent: ::core::option::Option<Parent>,
    #[prost(message, optional, tag = "3")]
    pub name: ::core::option::Option<OsStr>,
    #[prost(message, optional, tag = "4")]
    pub mode: ::core::option::Option<Mode>,
    #[prost(message, optional, tag = "5")]
    pub umask: ::core::option::Option<Umask>,
    #[prost(message, optional, tag = "6")]
    pub flags: ::core::option::Option<Flags>,
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
    #[prost(message, optional, tag = "2")]
    pub parent: ::core::option::Option<Parent>,
    #[prost(message, optional, tag = "3")]
    pub name: ::core::option::Option<OsStr>,
}
#[doc = r" Generated client implementations."]
pub mod keeper_work_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct KeeperWorkClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl KeeperWorkClient<tonic::transport::Channel> {
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
    impl<T> KeeperWorkClient<T>
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
        ) -> KeeperWorkClient<InterceptedService<T, F>>
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
            KeeperWorkClient::new(InterceptedService::new(inner, interceptor))
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
        ) -> Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.KeeperWork/read");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn write(
            &mut self,
            request: impl tonic::IntoRequest<super::Write>,
        ) -> Result<tonic::Response<super::ReplyWrite>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.KeeperWork/write");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn lookup(
            &mut self,
            request: impl tonic::IntoRequest<super::LookUp>,
        ) -> Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.KeeperWork/lookup");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create(
            &mut self,
            request: impl tonic::IntoRequest<super::Create>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.KeeperWork/create");
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
            let path = http::uri::PathAndQuery::from_static("/disfuser.KeeperWork/unlink");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod keeper_work_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with KeeperWorkServer."]
    #[async_trait]
    pub trait KeeperWork: Send + Sync + 'static {
        async fn read(
            &self,
            request: tonic::Request<super::Read>,
        ) -> Result<tonic::Response<super::Reply>, tonic::Status>;
        async fn write(
            &self,
            request: tonic::Request<super::Write>,
        ) -> Result<tonic::Response<super::ReplyWrite>, tonic::Status>;
        async fn lookup(
            &self,
            request: tonic::Request<super::LookUp>,
        ) -> Result<tonic::Response<super::Reply>, tonic::Status>;
        async fn create(
            &self,
            request: tonic::Request<super::Create>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status>;
        async fn unlink(
            &self,
            request: tonic::Request<super::Unlink>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct KeeperWorkServer<T: KeeperWork> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: KeeperWork> KeeperWorkServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for KeeperWorkServer<T>
    where
        T: KeeperWork,
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
                "/disfuser.KeeperWork/read" => {
                    #[allow(non_camel_case_types)]
                    struct readSvc<T: KeeperWork>(pub Arc<T>);
                    impl<T: KeeperWork> tonic::server::UnaryService<super::Read> for readSvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
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
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/disfuser.KeeperWork/write" => {
                    #[allow(non_camel_case_types)]
                    struct writeSvc<T: KeeperWork>(pub Arc<T>);
                    impl<T: KeeperWork> tonic::server::UnaryService<super::Write> for writeSvc<T> {
                        type Response = super::ReplyWrite;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Write>) -> Self::Future {
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
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/disfuser.KeeperWork/lookup" => {
                    #[allow(non_camel_case_types)]
                    struct lookupSvc<T: KeeperWork>(pub Arc<T>);
                    impl<T: KeeperWork> tonic::server::UnaryService<super::LookUp> for lookupSvc<T> {
                        type Response = super::Reply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
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
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/disfuser.KeeperWork/create" => {
                    #[allow(non_camel_case_types)]
                    struct createSvc<T: KeeperWork>(pub Arc<T>);
                    impl<T: KeeperWork> tonic::server::UnaryService<super::Create> for createSvc<T> {
                        type Response = super::Bool;
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
                "/disfuser.KeeperWork/unlink" => {
                    #[allow(non_camel_case_types)]
                    struct unlinkSvc<T: KeeperWork>(pub Arc<T>);
                    impl<T: KeeperWork> tonic::server::UnaryService<super::Unlink> for unlinkSvc<T> {
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
    impl<T: KeeperWork> Clone for KeeperWorkServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: KeeperWork> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: KeeperWork> tonic::transport::NamedService for KeeperWorkServer<T> {
        const NAME: &'static str = "disfuser.KeeperWork";
    }
}
