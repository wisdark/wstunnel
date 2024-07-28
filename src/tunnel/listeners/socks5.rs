use crate::protocols::socks5;
use crate::protocols::socks5::{Socks5Listener, Socks5Stream};
use crate::tunnel::RemoteAddr;
use anyhow::{anyhow, Context};
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{ready, Poll};
use std::time::Duration;
use tokio::io::{ReadHalf, WriteHalf};
use tokio_stream::Stream;

pub struct Socks5TunnelListener {
    listener: Socks5Listener,
}

impl Socks5TunnelListener {
    pub async fn new(
        bind_addr: SocketAddr,
        timeout: Option<Duration>,
        credentials: Option<(String, String)>,
    ) -> anyhow::Result<Self> {
        let listener = socks5::run_server(bind_addr, timeout, credentials)
            .await
            .with_context(|| anyhow!("Cannot start Socks5 server on {}", bind_addr))?;

        Ok(Self { listener })
    }
}

impl Stream for Socks5TunnelListener {
    type Item = anyhow::Result<((ReadHalf<Socks5Stream>, WriteHalf<Socks5Stream>), RemoteAddr)>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let ret = ready!(Pin::new(&mut this.listener).poll_next(cx));
        // TODO: Check if tokio::io::split can be avoided
        let ret = match ret {
            Some(Ok((stream, (host, port)))) => {
                let protocol = stream.local_protocol();
                Some(anyhow::Ok((tokio::io::split(stream), RemoteAddr { protocol, host, port })))
            }
            Some(Err(err)) => Some(Err(err)),
            None => None,
        };
        Poll::Ready(ret)
    }
}
