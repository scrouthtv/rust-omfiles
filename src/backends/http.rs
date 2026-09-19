//! Async reader backend for files served over HTTP(S), e.g. from S3 or other
//! object storage.
//!
//! This backend uses HTTP `Range` requests, so combined with
//! [`crate::reader_async::OmFileReaderAsync`] only the byte ranges required to
//! satisfy a given read (index/LUT blocks and the relevant compressed data
//! chunks) are ever transferred over the network.

use std::io::Read;

use crate::errors::OmFilesError;
use crate::traits::OmFileReaderBackendAsync;

/// An async backend that reads OM files directly from an HTTP(S) URL using
/// range requests.
///
/// Requires the remote server to support `Range` requests
/// (`Accept-Ranges: bytes`), which is the case for e.g. AWS S3.
pub struct HttpBackend {
    url: String,
    agent: ureq::Agent,
    file_size: u64,
}

impl HttpBackend {
    /// Creates a new HTTP backend for the given URL.
    ///
    /// This issues a `HEAD` request to determine the total size of the remote file.
    pub async fn new(url: impl Into<String>) -> Result<Self, OmFilesError> {
        let url = url.into();
        let agent = ureq::Agent::new_with_defaults();

        let agent_clone = agent.clone();
        let url_clone = url.clone();
        let file_size = blocking::unblock(move || Self::fetch_content_length(&agent_clone, &url_clone))
            .await?;

        Ok(Self {
            url,
            agent,
            file_size,
        })
    }

    fn fetch_content_length(agent: &ureq::Agent, url: &str) -> Result<u64, OmFilesError> {
        let response = agent
            .head(url)
            .call()
            .map_err(|e| OmFilesError::GenericError(format!("HTTP HEAD request failed: {e}")))?;

        response
            .headers()
            .get("Content-Length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| {
                OmFilesError::GenericError(
                    "HTTP response is missing a numeric Content-Length header".to_string(),
                )
            })
    }

    fn fetch_range(agent: &ureq::Agent, url: &str, offset: u64, count: u64) -> Result<Vec<u8>, OmFilesError> {
        let range_header = format!("bytes={}-{}", offset, offset + count - 1);

        let mut response = agent
            .get(url)
            .header("Range", &range_header)
            .call()
            .map_err(|e| OmFilesError::GenericError(format!("HTTP GET request failed: {e}")))?;

        let mut buffer = Vec::with_capacity(count as usize);
        response
            .body_mut()
            .as_reader()
            .read_to_end(&mut buffer)
            .map_err(|e| OmFilesError::GenericError(format!("Failed to read HTTP response body: {e}")))?;

        Ok(buffer)
    }
}

impl OmFileReaderBackendAsync for HttpBackend {
    type Bytes = Vec<u8>;

    fn count_async(&self) -> usize {
        self.file_size as usize
    }

    async fn get_bytes_async(&self, offset: u64, count: u64) -> Result<Self::Bytes, OmFilesError> {
        let agent = self.agent.clone();
        let url = self.url.clone();
        blocking::unblock(move || Self::fetch_range(&agent, &url, offset, count)).await
    }
}
