#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FRequest {
    #[prost(uint32, required, tag = "1")]
    pub uid: u32,
    #[prost(uint32, required, tag = "2")]
    pub gid: u32,
    #[prost(uint32, required, tag = "3")]
    pub pid: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Read {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(uint64, required, tag = "3")]
    pub fh: u64,
    #[prost(int64, required, tag = "4")]
    pub offset: i64,
    #[prost(uint32, required, tag = "5")]
    pub size: u32,
    #[prost(int32, required, tag = "6")]
    pub flags: i32,
    #[prost(uint64, optional, tag = "7")]
    pub lock_owner: ::core::option::Option<u64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    /// \[u8\]
    #[prost(uint32, required, tag = "1")]
    pub data: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Write {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(uint64, required, tag = "3")]
    pub fh: u64,
    #[prost(int64, required, tag = "4")]
    pub offset: i64,
    /// \[u8\]
    #[prost(string, required, tag = "5")]
    pub data: ::prost::alloc::string::String,
    #[prost(uint32, required, tag = "6")]
    pub write_flag: u32,
    #[prost(int32, required, tag = "7")]
    pub flags: i32,
    #[prost(uint64, optional, tag = "8")]
    pub lock_owner: ::core::option::Option<u64>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LookUp {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub parent: u64,
    /// OsStr name = 3;
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Create {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub parent: u64,
    /// OsStr name = 3;
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint32, required, tag = "4")]
    pub mode: u32,
    #[prost(uint32, required, tag = "5")]
    pub umask: u32,
    #[prost(int32, required, tag = "6")]
    pub flags: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Unlink {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub parent: u64,
    /// OsStr name = 3;
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Getattr {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Open {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(int32, required, tag = "3")]
    pub flags: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Release {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(uint64, required, tag = "3")]
    pub fh: u64,
    #[prost(int32, required, tag = "4")]
    pub flags: i32,
    #[prost(uint64, optional, tag = "5")]
    pub lock_owner: ::core::option::Option<u64>,
    #[prost(bool, required, tag = "6")]
    pub flush: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Setxattr {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, required, tag = "4")]
    pub value: ::prost::alloc::string::String,
    #[prost(int32, required, tag = "5")]
    pub flags: i32,
    #[prost(uint32, required, tag = "6")]
    pub position: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Getxattr {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint32, required, tag = "4")]
    pub size: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Listxattr {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(uint32, required, tag = "3")]
    pub size: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Access {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(int32, required, tag = "3")]
    pub mask: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Rename {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub parent: u64,
    #[prost(string, required, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint64, required, tag = "4")]
    pub newparent: u64,
    #[prost(string, required, tag = "5")]
    pub newname: ::prost::alloc::string::String,
    #[prost(uint32, required, tag = "6")]
    pub flags: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Setattr {
    #[prost(message, required, tag = "1")]
    pub frequest: FRequest,
    #[prost(uint64, required, tag = "2")]
    pub ino: u64,
    #[prost(uint32, optional, tag = "3")]
    pub mode: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag = "4")]
    pub uid: ::core::option::Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pub gid: ::core::option::Option<u32>,
    #[prost(uint64, optional, tag = "6")]
    pub size: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag = "7")]
    pub fh: ::core::option::Option<u64>,
    #[prost(uint32, optional, tag = "8")]
    pub flags: ::core::option::Option<u32>,
    #[prost(int64, optional, tag = "9")]
    pub atime_secs: ::core::option::Option<i64>,
    #[prost(uint32, optional, tag = "10")]
    pub atime_nsecs: ::core::option::Option<u32>,
    #[prost(int64, optional, tag = "11")]
    pub mtime_secs: ::core::option::Option<i64>,
    #[prost(uint32, optional, tag = "12")]
    pub mtime_nsecs: ::core::option::Option<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reply {
    /// \[u8\] or fileAttr string
    #[prost(string, required, tag = "1")]
    pub message: ::prost::alloc::string::String,
    /// for the Err when unwrap
    #[prost(int32, required, tag = "2")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteReply {
    #[prost(uint32, required, tag = "1")]
    pub size: u32,
    #[prost(int32, required, tag = "2")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateReply {
    #[prost(string, required, tag = "1")]
    pub file_attr: ::prost::alloc::string::String,
    #[prost(uint64, required, tag = "2")]
    pub fh: u64,
    #[prost(int32, required, tag = "3")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnlinkReply {
    #[prost(int32, required, tag = "1")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetattrReply {
    #[prost(string, required, tag = "1")]
    pub file_attr: ::prost::alloc::string::String,
    #[prost(int32, required, tag = "2")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OpenReply {
    #[prost(uint64, required, tag = "1")]
    pub fh: u64,
    #[prost(uint32, required, tag = "2")]
    pub openflag: u32,
    #[prost(int32, required, tag = "3")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReleaseReply {
    #[prost(int32, required, tag = "1")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetxattrReply {
    #[prost(int32, required, tag = "1")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetxattrReply {
    #[prost(string, required, tag = "1")]
    pub data: ::prost::alloc::string::String,
    #[prost(uint32, required, tag = "2")]
    pub size: u32,
    #[prost(int32, required, tag = "3")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListxattrReply {
    #[prost(string, required, tag = "1")]
    pub data: ::prost::alloc::string::String,
    #[prost(uint32, required, tag = "2")]
    pub size: u32,
    #[prost(int32, required, tag = "3")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccessReply {
    #[prost(int32, required, tag = "1")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RenameReply {
    #[prost(int32, required, tag = "1")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetattrReply {
    #[prost(string, required, tag = "1")]
    pub file_attr: ::prost::alloc::string::String,
    #[prost(int32, required, tag = "2")]
    pub errcode: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValue {
    #[prost(string, required, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pattern {
    #[prost(string, required, tag = "1")]
    pub prefix: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub suffix: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bool {
    #[prost(bool, required, tag = "1")]
    pub value: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Key {
    #[prost(string, required, tag = "1")]
    pub key: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(string, required, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StringList {
    #[prost(string, repeated, tag = "1")]
    pub list: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Clock {
    #[prost(uint64, required, tag = "1")]
    pub timestamp: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRemoveResponse {
    #[prost(uint32, required, tag = "1")]
    pub removed: u32,
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
        ) -> Result<tonic::Response<super::WriteReply>, tonic::Status> {
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
        ) -> Result<tonic::Response<super::Reply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/lookup");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn create(
            &mut self,
            request: impl tonic::IntoRequest<super::Create>,
        ) -> Result<tonic::Response<super::CreateReply>, tonic::Status> {
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
        ) -> Result<tonic::Response<super::UnlinkReply>, tonic::Status> {
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
        pub async fn open(
            &mut self,
            request: impl tonic::IntoRequest<super::Open>,
        ) -> Result<tonic::Response<super::OpenReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/open");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn getattr(
            &mut self,
            request: impl tonic::IntoRequest<super::Getattr>,
        ) -> Result<tonic::Response<super::GetattrReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/getattr");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn access(
            &mut self,
            request: impl tonic::IntoRequest<super::Access>,
        ) -> Result<tonic::Response<super::AccessReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/access");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn rename(
            &mut self,
            request: impl tonic::IntoRequest<super::Rename>,
        ) -> Result<tonic::Response<super::RenameReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/rename");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn release(
            &mut self,
            request: impl tonic::IntoRequest<super::Release>,
        ) -> Result<tonic::Response<super::ReleaseReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/release");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn setxattr(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::Setxattr>,
        ) -> Result<tonic::Response<super::SetxattrReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/setxattr");
            self.inner
                .client_streaming(request.into_streaming_request(), path, codec)
                .await
        }
        pub async fn getxattr(
            &mut self,
            request: impl tonic::IntoRequest<super::Getxattr>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::GetxattrReply>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/getxattr");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn listxattr(
            &mut self,
            request: impl tonic::IntoRequest<super::Listxattr>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::ListxattrReply>>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/listxattr");
            self.inner
                .server_streaming(request.into_request(), path, codec)
                .await
        }
        pub async fn setattr(
            &mut self,
            request: impl tonic::IntoRequest<super::Setattr>,
        ) -> Result<tonic::Response<super::SetattrReply>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/setattr");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get(
            &mut self,
            request: impl tonic::IntoRequest<super::Key>,
        ) -> Result<tonic::Response<super::Value>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/get");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn set(
            &mut self,
            request: impl tonic::IntoRequest<super::KeyValue>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/set");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn keys(
            &mut self,
            request: impl tonic::IntoRequest<super::Pattern>,
        ) -> Result<tonic::Response<super::StringList>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/keys");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_get(
            &mut self,
            request: impl tonic::IntoRequest<super::Key>,
        ) -> Result<tonic::Response<super::StringList>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/listGet");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_append(
            &mut self,
            request: impl tonic::IntoRequest<super::KeyValue>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/listAppend");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_remove(
            &mut self,
            request: impl tonic::IntoRequest<super::KeyValue>,
        ) -> Result<tonic::Response<super::ListRemoveResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/listRemove");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn list_keys(
            &mut self,
            request: impl tonic::IntoRequest<super::Pattern>,
        ) -> Result<tonic::Response<super::StringList>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/listKeys");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn clock(
            &mut self,
            request: impl tonic::IntoRequest<super::Clock>,
        ) -> Result<tonic::Response<super::Clock>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/disfuser.disfuser/clock");
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
        ) -> Result<tonic::Response<super::WriteReply>, tonic::Status>;
        async fn lookup(
            &self,
            request: tonic::Request<super::LookUp>,
        ) -> Result<tonic::Response<super::Reply>, tonic::Status>;
        async fn create(
            &self,
            request: tonic::Request<super::Create>,
        ) -> Result<tonic::Response<super::CreateReply>, tonic::Status>;
        async fn unlink(
            &self,
            request: tonic::Request<super::Unlink>,
        ) -> Result<tonic::Response<super::UnlinkReply>, tonic::Status>;
        async fn open(
            &self,
            request: tonic::Request<super::Open>,
        ) -> Result<tonic::Response<super::OpenReply>, tonic::Status>;
        async fn getattr(
            &self,
            request: tonic::Request<super::Getattr>,
        ) -> Result<tonic::Response<super::GetattrReply>, tonic::Status>;
        async fn access(
            &self,
            request: tonic::Request<super::Access>,
        ) -> Result<tonic::Response<super::AccessReply>, tonic::Status>;
        async fn rename(
            &self,
            request: tonic::Request<super::Rename>,
        ) -> Result<tonic::Response<super::RenameReply>, tonic::Status>;
        async fn release(
            &self,
            request: tonic::Request<super::Release>,
        ) -> Result<tonic::Response<super::ReleaseReply>, tonic::Status>;
        async fn setxattr(
            &self,
            request: tonic::Request<tonic::Streaming<super::Setxattr>>,
        ) -> Result<tonic::Response<super::SetxattrReply>, tonic::Status>;
        #[doc = "Server streaming response type for the getxattr method."]
        type getxattrStream: futures_core::Stream<Item = Result<super::GetxattrReply, tonic::Status>>
            + Send
            + 'static;
        async fn getxattr(
            &self,
            request: tonic::Request<super::Getxattr>,
        ) -> Result<tonic::Response<Self::getxattrStream>, tonic::Status>;
        #[doc = "Server streaming response type for the listxattr method."]
        type listxattrStream: futures_core::Stream<Item = Result<super::ListxattrReply, tonic::Status>>
            + Send
            + 'static;
        async fn listxattr(
            &self,
            request: tonic::Request<super::Listxattr>,
        ) -> Result<tonic::Response<Self::listxattrStream>, tonic::Status>;
        async fn setattr(
            &self,
            request: tonic::Request<super::Setattr>,
        ) -> Result<tonic::Response<super::SetattrReply>, tonic::Status>;
        async fn get(
            &self,
            request: tonic::Request<super::Key>,
        ) -> Result<tonic::Response<super::Value>, tonic::Status>;
        async fn set(
            &self,
            request: tonic::Request<super::KeyValue>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status>;
        async fn keys(
            &self,
            request: tonic::Request<super::Pattern>,
        ) -> Result<tonic::Response<super::StringList>, tonic::Status>;
        async fn list_get(
            &self,
            request: tonic::Request<super::Key>,
        ) -> Result<tonic::Response<super::StringList>, tonic::Status>;
        async fn list_append(
            &self,
            request: tonic::Request<super::KeyValue>,
        ) -> Result<tonic::Response<super::Bool>, tonic::Status>;
        async fn list_remove(
            &self,
            request: tonic::Request<super::KeyValue>,
        ) -> Result<tonic::Response<super::ListRemoveResponse>, tonic::Status>;
        async fn list_keys(
            &self,
            request: tonic::Request<super::Pattern>,
        ) -> Result<tonic::Response<super::StringList>, tonic::Status>;
        async fn clock(
            &self,
            request: tonic::Request<super::Clock>,
        ) -> Result<tonic::Response<super::Clock>, tonic::Status>;
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
                        type Response = super::WriteReply;
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
                    impl<T: Disfuser> tonic::server::UnaryService<super::LookUp> for lookupSvc<T> {
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
                "/disfuser.disfuser/create" => {
                    #[allow(non_camel_case_types)]
                    struct createSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Create> for createSvc<T> {
                        type Response = super::CreateReply;
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
                        type Response = super::UnlinkReply;
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
                "/disfuser.disfuser/open" => {
                    #[allow(non_camel_case_types)]
                    struct openSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Open> for openSvc<T> {
                        type Response = super::OpenReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Open>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).open(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = openSvc(inner);
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
                "/disfuser.disfuser/getattr" => {
                    #[allow(non_camel_case_types)]
                    struct getattrSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Getattr> for getattrSvc<T> {
                        type Response = super::GetattrReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Getattr>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).getattr(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = getattrSvc(inner);
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
                "/disfuser.disfuser/access" => {
                    #[allow(non_camel_case_types)]
                    struct accessSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Access> for accessSvc<T> {
                        type Response = super::AccessReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Access>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).access(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = accessSvc(inner);
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
                "/disfuser.disfuser/rename" => {
                    #[allow(non_camel_case_types)]
                    struct renameSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Rename> for renameSvc<T> {
                        type Response = super::RenameReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Rename>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).rename(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = renameSvc(inner);
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
                "/disfuser.disfuser/release" => {
                    #[allow(non_camel_case_types)]
                    struct releaseSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Release> for releaseSvc<T> {
                        type Response = super::ReleaseReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Release>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).release(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = releaseSvc(inner);
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
                "/disfuser.disfuser/setxattr" => {
                    #[allow(non_camel_case_types)]
                    struct setxattrSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::ClientStreamingService<super::Setxattr> for setxattrSvc<T> {
                        type Response = super::SetxattrReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::Setxattr>>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).setxattr(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = setxattrSvc(inner);
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
                "/disfuser.disfuser/getxattr" => {
                    #[allow(non_camel_case_types)]
                    struct getxattrSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::ServerStreamingService<super::Getxattr> for getxattrSvc<T> {
                        type Response = super::GetxattrReply;
                        type ResponseStream = T::getxattrStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Getxattr>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).getxattr(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = getxattrSvc(inner);
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
                "/disfuser.disfuser/listxattr" => {
                    #[allow(non_camel_case_types)]
                    struct listxattrSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::ServerStreamingService<super::Listxattr> for listxattrSvc<T> {
                        type Response = super::ListxattrReply;
                        type ResponseStream = T::listxattrStream;
                        type Future =
                            BoxFuture<tonic::Response<Self::ResponseStream>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Listxattr>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).listxattr(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = listxattrSvc(inner);
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
                "/disfuser.disfuser/setattr" => {
                    #[allow(non_camel_case_types)]
                    struct setattrSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Setattr> for setattrSvc<T> {
                        type Response = super::SetattrReply;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Setattr>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).setattr(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = setattrSvc(inner);
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
                "/disfuser.disfuser/get" => {
                    #[allow(non_camel_case_types)]
                    struct getSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Key> for getSvc<T> {
                        type Response = super::Value;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Key>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = getSvc(inner);
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
                "/disfuser.disfuser/set" => {
                    #[allow(non_camel_case_types)]
                    struct setSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::KeyValue> for setSvc<T> {
                        type Response = super::Bool;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::KeyValue>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).set(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = setSvc(inner);
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
                "/disfuser.disfuser/keys" => {
                    #[allow(non_camel_case_types)]
                    struct keysSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Pattern> for keysSvc<T> {
                        type Response = super::StringList;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Pattern>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).keys(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = keysSvc(inner);
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
                "/disfuser.disfuser/listGet" => {
                    #[allow(non_camel_case_types)]
                    struct listGetSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Key> for listGetSvc<T> {
                        type Response = super::StringList;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Key>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_get(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = listGetSvc(inner);
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
                "/disfuser.disfuser/listAppend" => {
                    #[allow(non_camel_case_types)]
                    struct listAppendSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::KeyValue> for listAppendSvc<T> {
                        type Response = super::Bool;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::KeyValue>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_append(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = listAppendSvc(inner);
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
                "/disfuser.disfuser/listRemove" => {
                    #[allow(non_camel_case_types)]
                    struct listRemoveSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::KeyValue> for listRemoveSvc<T> {
                        type Response = super::ListRemoveResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::KeyValue>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_remove(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = listRemoveSvc(inner);
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
                "/disfuser.disfuser/listKeys" => {
                    #[allow(non_camel_case_types)]
                    struct listKeysSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Pattern> for listKeysSvc<T> {
                        type Response = super::StringList;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Pattern>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).list_keys(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = listKeysSvc(inner);
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
                "/disfuser.disfuser/clock" => {
                    #[allow(non_camel_case_types)]
                    struct clockSvc<T: Disfuser>(pub Arc<T>);
                    impl<T: Disfuser> tonic::server::UnaryService<super::Clock> for clockSvc<T> {
                        type Response = super::Clock;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Clock>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).clock(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = clockSvc(inner);
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
