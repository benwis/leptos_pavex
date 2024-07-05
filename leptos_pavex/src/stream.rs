use bytes::Bytes;
use futures::Stream;
use http_body::Frame;
use pavex::response::body::raw::RawBody;
use pin_project::pin_project;
use std::pin::Pin;
use std::error::Error;

/// A stream wrapper so that we may implement Pavex's RawBody for a stream. This one is used generically 
/// in the Res/Req machinery
#[pin_project]
pub struct PavexStream<S>
where
S: Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync + 'static >>>,
{
    #[pin]
    pub inner: S,
}

impl<S> PavexStream<S>
where
    S: Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync + 'static>>>,
{
    pub fn to_inner_pin(self: Pin<&mut Self>) -> Pin<&mut S> {
        let this = self.project();
        this.inner
    }
}

impl<S> RawBody for PavexStream<S>
where
    S: Stream<Item = Result<Bytes, Box<dyn Error + Send + Sync + 'static>>>,
{
    type Data = Bytes;

    type Error = Box<dyn Error + Send + Sync + 'static>;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let stream: Pin<&mut S> = self.to_inner_pin();

        S::poll_next(stream, cx).map(|o| o.map(|r| r.map_err(|e| e.into()).map(Frame::data)))
    }
}

/// A stream wrapper specifically for the Leptos HTML output stream so that we may implement Pavex's RawBody trait for it
#[pin_project]
pub struct LeptosPavexStream<S>
where
    S: Stream<Item = Result<String, std::io::Error>>,
{
    #[pin]
    pub inner: S,
}

impl<S> LeptosPavexStream<S>
where
    S: Stream<Item = Result<String, std::io::Error>>,
{
    pub fn to_inner_pin(self: Pin<&mut Self>) -> Pin<&mut S> {
        let this = self.project();
        this.inner
    }
}

impl<S> RawBody for LeptosPavexStream<S>
where
    S: Stream<Item = Result<String, std::io::Error>>,
{
    type Data = Bytes;

    type Error = std::io::Error;

    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let stream: Pin<&mut S> = self.to_inner_pin();

        S::poll_next(stream, cx).map(|o| o.map(|r| r.map(|d| Frame::data(Bytes::from(d)))))
    }
}
