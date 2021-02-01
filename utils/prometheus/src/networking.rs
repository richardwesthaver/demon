use std::task::{Context, Poll};

use async_std::pin::Pin;
use futures_util::{
  io::{AsyncRead, AsyncWrite},
  stream::Stream,
};

pub struct Incoming<'a>(pub async_std::net::Incoming<'a>);

impl hyper::server::accept::Accept for Incoming<'_> {
  type Conn = TcpStream;
  type Error = async_std::io::Error;

  fn poll_accept(
    self: Pin<&mut Self>,
    cx: &mut Context,
  ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
    Pin::new(&mut Pin::into_inner(self).0)
      .poll_next(cx)
      .map(|opt| opt.map(|res| res.map(TcpStream)))
  }
}

pub struct TcpStream(pub async_std::net::TcpStream);

impl tokio::io::AsyncRead for TcpStream {
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &mut [u8],
  ) -> Poll<Result<usize, std::io::Error>> {
    Pin::new(&mut Pin::into_inner(self).0).poll_read(cx, buf)
  }
}

impl tokio::io::AsyncWrite for TcpStream {
  fn poll_write(
    self: Pin<&mut Self>,
    cx: &mut Context,
    buf: &[u8],
  ) -> Poll<Result<usize, std::io::Error>> {
    Pin::new(&mut Pin::into_inner(self).0).poll_write(cx, buf)
  }

  fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), std::io::Error>> {
    Pin::new(&mut Pin::into_inner(self).0).poll_flush(cx)
  }

  fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), std::io::Error>> {
    Pin::new(&mut Pin::into_inner(self).0).poll_close(cx)
  }
}
