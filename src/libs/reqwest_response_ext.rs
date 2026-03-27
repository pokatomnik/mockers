use anyhow::Context;
use bytes::{Bytes, BytesMut};
use futures_util::TryStreamExt;

pub(crate) trait ReqwestResponseExt {
    const DEFAULT_BUFFER_ALLOC_SIZE: usize;

    async fn read_until_cap(
        self,
        max_bytes: usize,
        init_buf_size: Option<usize>,
    ) -> anyhow::Result<Bytes>;
}

impl ReqwestResponseExt for reqwest::Response {
    /// This value is considered a fair default for most cases.
    const DEFAULT_BUFFER_ALLOC_SIZE: usize = 1024 * 8;

    /// Reads the response body into a [`Bytes`] buffer, enforcing a maximum size limit.
    ///
    /// This method streams the response body in chunks to avoid loading the entire
    /// body into memory at once, but stops and returns an error if the body size
    /// exceeds the specified `max_bytes` limit.
    ///
    /// # Arguments
    ///
    /// * `max_bytes` - The maximum allowed size of the response body in bytes.
    /// * `init_buf_size` - An optional initial capacity for the internal buffer.
    ///   If `None`, defaults to [`Self::DEFAULT_BUFFER_ALLOC_SIZE`].
    ///
    /// # Returns
    ///
    /// A `Result` containing the buffered bytes on success, or an error if the
    /// response body exceeds the limit or if a stream read fails.
    async fn read_until_cap(
        self,
        max_bytes: usize,
        init_buf_size: Option<usize>,
    ) -> anyhow::Result<Bytes> {
        let mut buf =
            BytesMut::with_capacity(init_buf_size.unwrap_or(Self::DEFAULT_BUFFER_ALLOC_SIZE));
        let mut stream = self.bytes_stream();
        while let Some(chunk) = stream
            .try_next()
            .await
            .context("failed to read upstream response chunk")?
        {
            if chunk.len() > max_bytes - buf.len() {
                return Err(anyhow::anyhow!(
                    "upstream response exceeded limit of {max_bytes} bytes"
                ));
            }
            buf.extend_from_slice(&chunk);
        }
        Ok(buf.freeze())
    }
}
