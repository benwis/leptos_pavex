use std::error::Error;
use std::{fmt::Display, fmt::Debug, str::FromStr};
use std::pin::Pin;
use futures::Stream;
use http_body::Frame;
use leptos::server_fn::error::ServerFnErrorErr;
use leptos::server_fn::ServerFnError;
use pavex::response::body::raw::RawBody;
use bytes::Bytes;
use pin_project::pin_project;


#[pin_project]
   pub struct PavexStream<S, CustErr>
   where 
        S: Stream<Item = Result<Bytes, ServerFnError<CustErr>>>,
        CustErr: Send + Sync + Debug + FromStr + Display + 'static{
    #[pin]
    pub inner: S,
   }



impl<S, CustErr> PavexStream<S,CustErr>
    where 
        S: Stream<Item = Result<Bytes, ServerFnError<CustErr>>>,
        CustErr: Send + Sync + Debug + FromStr + Display  +  'static 
    {
pub fn to_inner_pin(self: Pin<&mut Self>)-> Pin<&mut S>{
    let this=self.project();
    this.inner
}
}

impl<S, CustErr> RawBody for PavexStream<S,CustErr>
where 
    S: Stream<Item = Result<Bytes, ServerFnError<CustErr>>>, 
    CustErr: Send + Sync + Debug + FromStr + Display + 'static,
    ServerFnError<CustErr>: From<ServerFnErrorErr<CustErr>> {
    type Data = Bytes;
    
    type Error = ServerFnError<CustErr>;
    
    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let stream:Pin<&mut S> = self.to_inner_pin();
        
        S::poll_next(stream, cx)
    .map(|o| o.map(|r| r.map_err(|e| e.into()).map(Frame::data)))
    }
}