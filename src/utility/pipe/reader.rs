use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::Receiver;

pub struct PipeReader<'a> {
    pub(crate) rx: Receiver<&'a [u8]>,
}

impl<'a> PipeReader<'a> {
    pub fn new(rx: Receiver<&'a [u8]>) -> PipeReader<'a> {
        PipeReader { rx }
    }
}
