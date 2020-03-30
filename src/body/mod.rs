use crate::prelude::*;
use hyper::body::Buf;
use hyper::{body::HttpBody, header::HeaderValue, HeaderMap};
use pin_project_lite::pin_project;
use std::io::BufReader;
use std::marker::PhantomData;
use std::marker::Unpin;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::AsyncRead;

pub const DEFAULT_BUF_SIZE: usize = 8 * 1024;

pub struct Buffer {
    ptr: *const u8,
    len: usize,
    pos: usize,
}

unsafe impl std::marker::Send for Buffer {}

impl Buf for Buffer {
    fn remaining(&self) -> usize {
        self.len - self.pos
    }

    fn bytes(&self) -> &[u8] {
        println!("Called bytes");
        unsafe { std::slice::from_raw_parts(self.ptr, self.len - self.pos) }
    }

    fn advance(&mut self, cnt: usize) {
        self.pos += cnt;
        unsafe {
            self.ptr = self.ptr.add(cnt);
        }
    }
}

pin_project! {
    pub struct StreamBody<R> {
        #[pin]
        pub(crate) reader: R,
        pub(crate) buf: Box<[u8]>,
        pub(crate) len: usize,
    }
}

impl<R> StreamBody<R>
where
    R: AsyncRead + Send + Sync,
{
    pub fn new(reader: R) -> StreamBody<R> {
        StreamBody::with_capacity(DEFAULT_BUF_SIZE, reader)
    }

    pub fn with_capacity(capacity: usize, reader: R) -> StreamBody<R> {
        unsafe {
            let mut buffer = Vec::with_capacity(capacity);
            buffer.set_len(capacity);

            {
                let b = &mut *(&mut buffer[..] as *mut [u8] as *mut [MaybeUninit<u8>]);
                reader.prepare_uninitialized_buffer(b);
            }

            StreamBody {
                reader,
                buf: buffer.into_boxed_slice(),
                len: 0,
            }
        }
    }
}

impl<R> HttpBody for StreamBody<R>
where
    R: AsyncRead + Send + Sync + Unpin,
{
    type Data = Buffer;
    type Error = crate::Error;

    fn poll_data(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let me = self.as_mut().project();

        // let buf = &mut self.buf;
        let poll_status = me.reader.poll_read(cx, me.buf);
        // let poll_status = Pin::new(&mut self.reader).poll_read(cx, &mut buf[..]);

        match poll_status {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(result) => match result {
                Ok(read_count) => {
                    if read_count > 0 {
                        let buf = &self.buf;
                        let buffer = Buffer {
                            ptr: buf.as_ptr(),
                            len: read_count,
                            pos: 0,
                        };
                        return Poll::Ready(Some(Ok(buffer)));
                    } else {
                        return Poll::Ready(None);
                    }
                }
                Err(err) => return Poll::Ready(Some(Err(err.wrap()))),
            },
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Option<HeaderMap<HeaderValue>>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}
