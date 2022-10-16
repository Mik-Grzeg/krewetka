#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlowMessageClassBatched {
    #[prost(message, repeated, tag="1")]
    pub classifications: ::prost::alloc::vec::Vec<FlowMessageClass>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlowMessageBatched {
    #[prost(message, repeated, tag="1")]
    pub messages: ::prost::alloc::vec::Vec<FlowMessage>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlowMessageClass {
    #[prost(bool, tag="1")]
    pub malicious: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlowMessage {
    #[prost(uint64, tag="1")]
    pub out_bytes: u64,
    #[prost(uint64, tag="2")]
    pub out_pkts: u64,
    /// In sizes of packets
    #[prost(uint64, tag="3")]
    pub in_bytes: u64,
    #[prost(uint64, tag="4")]
    pub in_pkts: u64,
    /// Source/destination addresses
    #[prost(string, tag="5")]
    pub ipv4_src_addr: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub ipv4_dst_addr: ::prost::alloc::string::String,
    /// Layer 7 protocol
    #[prost(float, tag="7")]
    pub l7_proto: f32,
    /// Layer 4 port
    #[prost(uint32, tag="8")]
    pub l4_dst_port: u32,
    #[prost(uint32, tag="9")]
    pub l4_src_port: u32,
    /// Duration
    #[prost(uint64, tag="10")]
    pub flow_duration_milliseconds: u64,
    /// Protocol
    #[prost(uint32, tag="11")]
    pub protocol: u32,
    /// TCP flags
    #[prost(uint32, tag="12")]
    pub tcp_flags: u32,
}
/// Generated client implementations.
pub mod flow_message_classifier_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct FlowMessageClassifierClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl FlowMessageClassifierClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> FlowMessageClassifierClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> FlowMessageClassifierClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            FlowMessageClassifierClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        pub async fn classify_streaming(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::FlowMessage>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::FlowMessageClass>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/flow.FlowMessageClassifier/ClassifyStreaming",
            );
            self.inner.streaming(request.into_streaming_request(), path, codec).await
        }
        pub async fn classify_batch(
            &mut self,
            request: impl tonic::IntoRequest<super::FlowMessageBatched>,
        ) -> Result<tonic::Response<super::FlowMessageClassBatched>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/flow.FlowMessageClassifier/ClassifyBatch",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn classify(
            &mut self,
            request: impl tonic::IntoRequest<super::FlowMessage>,
        ) -> Result<tonic::Response<super::FlowMessageClass>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/flow.FlowMessageClassifier/Classify",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
