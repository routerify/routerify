use tokio::sync::mpsc;

pub use reader::PipeReader;
pub use writer::PipeWriter;

mod reader;
mod writer;

pub fn pipe<'a>() -> (PipeWriter<'a>, PipeReader<'a>) {
    let (tx, rx) = mpsc::channel(1);
    (PipeWriter::new(tx), PipeReader::new(rx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipe() {
        let (r, w) = pipe();
    }
}
