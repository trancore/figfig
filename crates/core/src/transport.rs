use crate::MAX_FRAME;
use anyhow::{bail, Result};
use serde::Serialize;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

/// Serialize one complete JSON line before writing any bytes to the stream.
pub async fn write_frame(
    writer: &mut (impl AsyncWrite + Unpin),
    message: &impl Serialize,
) -> Result<()> {
    let mut frame = serde_json::to_vec(message)?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME {
        bail!("Protocol frame too large");
    }
    writer.write_all(&frame).await?;
    Ok(())
}

/// The partial frame belongs to the reader, so select! cancellation cannot lose bytes.
pub struct FrameReader<R> {
    reader: R,
    partial: Vec<u8>,
}

impl<R: AsyncBufRead + Unpin> FrameReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            partial: Vec::new(),
        }
    }
    pub async fn next_frame(&mut self) -> Result<Option<Vec<u8>>> {
        loop {
            let available = self.reader.fill_buf().await?;
            if available.is_empty() {
                return if self.partial.is_empty() {
                    Ok(None)
                } else {
                    bail!("Truncated protocol frame")
                };
            }
            let end = available.iter().position(|&b| b == b'\n');
            let n = end.map(|i| i + 1).unwrap_or(available.len());
            if self.partial.len() + n > MAX_FRAME {
                bail!("Protocol frame too large");
            }
            self.partial.extend_from_slice(&available[..n]);
            self.reader.consume(n);
            if end.is_some() {
                return Ok(Some(std::mem::take(&mut self.partial)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn json_newlines_are_escaped_inside_a_single_frame() {
        let mut bytes = Vec::new();
        let message = serde_json::json!({ "buffer": "git 日\n" });
        write_frame(&mut bytes, &message).await.unwrap();
        assert_eq!(bytes.iter().filter(|&&byte| byte == b'\n').count(), 1);
        let mut reader = FrameReader::new(BufReader::new(bytes.as_slice()));
        let frame = reader.next_frame().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&frame).unwrap(),
            message
        );
        assert!(reader.next_frame().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn frame_limit_includes_the_delimiter_and_rejects_before_writing() {
        let mut bytes = Vec::new();
        let largest_message = "x".repeat(MAX_FRAME - 3); // JSON quotes and newline.
        write_frame(&mut bytes, &largest_message).await.unwrap();
        assert_eq!(bytes.len(), MAX_FRAME);
        let mut reader = FrameReader::new(BufReader::new(bytes.as_slice()));
        assert_eq!(reader.next_frame().await.unwrap().unwrap().len(), MAX_FRAME);
        bytes.clear();
        assert!(write_frame(&mut bytes, &(largest_message + "x"))
            .await
            .is_err());
        assert!(bytes.is_empty());
    }

    #[tokio::test]
    async fn partial_frame_survives_a_cancelled_read() {
        let (read, mut write) = tokio::io::duplex(100);
        let mut reader = FrameReader::new(BufReader::new(read));
        write.write_all(b"{\"type\":").await.unwrap();
        tokio::select! {
            biased;
            _ = reader.next_frame() => panic!("Frame must be incomplete"),
            _ = tokio::task::yield_now() => (),
        }
        write.write_all(b"\"status\"}\n").await.unwrap();
        assert_eq!(
            reader.next_frame().await.unwrap().unwrap(),
            b"{\"type\":\"status\"}\n"
        );
    }
}
