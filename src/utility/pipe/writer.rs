use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncWrite;
use tokio::sync::mpsc::Sender;

// pin_project! {
//     pub struct PipeWriter<'a> {
//         #[pin]
//         pub(crate) tx: Sender<&'a [u8]>,
//     }
// }

pub struct PipeWriter<'a> {
    pub(crate) tx: Sender<&'a [u8]>,
}

impl<'a> PipeWriter<'a> {
    pub fn new(tx: Sender<&'a [u8]>) -> PipeWriter<'a> {
        PipeWriter { tx }
    }
}

impl<'a> AsyncWrite for PipeWriter<'a> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let mut fut = self.tx.send(buf);

        // let p = Box::pin(fut).as_mut();

        // let p = Pin::new(&mut fut);
        // fut.po
        // let me = self.project();
        // let tx: Sender<&'a [u8]> = me.tx;

        // if buf.len() >= me.buf.capacity() {
        //     me.inner.poll_write(cx, buf)
        // } else {
        //     Poll::Ready(me.buf.write(buf))
        // }

        todo!();
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        // ready!(self.as_mut().flush_buf(cx))?;
        // self.get_pin_mut().poll_flush(cx)

        todo!();
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        // ready!(self.as_mut().flush_buf(cx))?;
        // self.get_pin_mut().poll_shutdown(cx)
        todo!();
    }
}
