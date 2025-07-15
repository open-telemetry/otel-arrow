//! crate containing GRPC Server implementations for the OTLP services that
//! convert the received OTLP signals into OTAP

use std::convert::Infallible;
use std::sync::Arc;
use std::task::Poll;

use async_trait::async_trait;
use futures::future::BoxFuture;
use http::{Request, Response};
use otel_arrow_rust::otap::OtapBatch;
use prost::bytes::Buf;
use tonic::body::Body;
use tonic::codec::{Codec, Decoder, Encoder};
use tonic::server::{Grpc, NamedService, UnaryService};
use tonic::Status;

use crate::encoder::encode_logs_otap_batch;
use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;


// just trying to satisfy the type system ... 
struct NoopEncoder {
    
}

impl Encoder for NoopEncoder {
    type Error = Status;
    type Item = ();

    fn encode(&mut self, item: Self::Item, dst: &mut tonic::codec::EncodeBuf<'_>) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct OtapBatchCodec {

}

impl Codec for OtapBatchCodec {
    type Decode = OtapBatch;
    type Encode = ();

    type Encoder = NoopEncoder;
    type Decoder = OtapBatchDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        NoopEncoder {}        
    }

    fn decoder(&mut self) -> Self::Decoder {
        OtapBatchDecoder {}
    }
}

struct OtapBatchDecoder {

}

impl Decoder for OtapBatchDecoder {
    type Item = OtapBatch;

    type Error = Status;

    fn decode(&mut self, src: &mut tonic::codec::DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {

        
        let buf = src.chunk();
        let view_impl = RawLogsData::new(buf);
        let res = encode_logs_otap_batch(&view_impl);
        src.advance(buf.len());
        match res {
            Ok(batch) => {
                println!("got batch wahoo!");
                Ok(Some(batch))
            },
            Err(e) => {
                println!("got error oh no: {:?}", e);
                // TODO add the real message
                Err(Status::internal("Internal Error"))
            }
        }
        // println!("I'm here 1");
        // Err(Status::unknown("booooobYYYYYY!!"))
    }
}

// TODO write docs
#[allow(missing_docs)]
#[async_trait]
pub trait OtapBatchService: Send + Sync + 'static {

   async fn export(&self, request: tonic::Request<OtapBatch>) -> Result<tonic::Response<()>, Status>;
}

struct BatchHandlerService<T: OtapBatchService>(Arc<T>);

impl<T> UnaryService<OtapBatch> for BatchHandlerService<T> where T: OtapBatchService {
    type Response = ();
    type Future = BoxFuture<'static, Result<tonic::Response<Self::Response>, Status>>;
    
    fn call(&mut self, request: tonic::Request<OtapBatch>) -> Self::Future {
        println!("I'm here 2");
        todo!()
    }
}


/// implementation of OTLP bytes -> OTAP GRPC server for logs
#[derive(Clone)]
pub struct LogsServiceServer<T: Clone> {
    /// TODO make this not pub
    pub inner: Arc<T>
}

impl<T> tower_service::Service<Request<Body>> for LogsServiceServer<T> 
where T: OtapBatchService + Clone + Send + Sync + 'static 
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        println!("boob");

        // todo!("AHHHH");

        let inner = self.inner.clone();
        Box::pin(async move {
            let codec = OtapBatchCodec {}; // TODO constructor
            let mut grpc = Grpc::new(codec); // TODO compression/max message size settings
            let svc = BatchHandlerService(inner);
            let res = grpc.unary(svc, req).await;
            Ok(res)
            // let body = req.body();
            // body.
            // let logs_view = RawLogsData::new()
            // let logs = encode_logs_otap_batch(logs_view)            

            // println!("other boob");
            // let mut response = Response::new(Body::default());
            
            // let headers = response.headers_mut();
            // let _ = headers.insert(
            //     Status::GRPC_STATUS,
            //     (tonic::Code::Internal as i32).into(),
            // );
            // let _ = headers.insert(
            //     http::header::CONTENT_TYPE,
            //     tonic::metadata::GRPC_CONTENT_TYPE,
            // );

            // Ok(response)
        })


    }

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
        // println!("boob2");
        // todo!("here 2")
    }
}

/// TODO maybe we should import this
pub const LOGS_SERVICE_NAME: &str = "opentelemetry.proto.collector.logs.v1.LogsService";

impl<T> NamedService for LogsServiceServer<T> where T: Clone {
    const NAME: &'static str = LOGS_SERVICE_NAME;
}