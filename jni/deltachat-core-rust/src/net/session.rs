use fast_socks5::client::Socks5Stream;
use std::pin::Pin;
use std::time::Duration;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, BufStream, BufWriter};
use tokio_io_timeout::TimeoutStream;

pub(crate) trait SessionStream:
    AsyncRead + AsyncWrite + Unpin + Send + Sync + std::fmt::Debug
{
    /// Change the read timeout on the session stream.
    fn set_read_timeout(&mut self, timeout: Option<Duration>);
}

impl SessionStream for Box<dyn SessionStream> {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.as_mut().set_read_timeout(timeout);
    }
}
impl<T: SessionStream> SessionStream for async_native_tls::TlsStream<T> {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.get_mut().set_read_timeout(timeout);
    }
}
impl<T: SessionStream> SessionStream for tokio_rustls::client::TlsStream<T> {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.get_mut().0.set_read_timeout(timeout);
    }
}
impl<T: SessionStream> SessionStream for BufStream<T> {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.get_mut().set_read_timeout(timeout);
    }
}
impl<T: SessionStream> SessionStream for BufWriter<T> {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.get_mut().set_read_timeout(timeout);
    }
}
impl<T: AsyncRead + AsyncWrite + Send + Sync + std::fmt::Debug> SessionStream
    for Pin<Box<TimeoutStream<T>>>
{
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.as_mut().set_read_timeout_pinned(timeout);
    }
}
impl<T: SessionStream> SessionStream for Socks5Stream<T> {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.get_socket_mut().set_read_timeout(timeout)
    }
}
impl<T: SessionStream> SessionStream for shadowsocks::ProxyClientStream<T> {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.get_mut().set_read_timeout(timeout)
    }
}
impl<T: SessionStream> SessionStream for async_imap::DeflateStream<T> {
    fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.get_mut().set_read_timeout(timeout)
    }
}

/// Session stream with a read buffer.
pub(crate) trait SessionBufStream: SessionStream + AsyncBufRead {}

impl<T: SessionStream + AsyncBufRead> SessionBufStream for T {}
