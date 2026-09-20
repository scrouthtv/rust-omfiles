//! Demonstrates reading a variable from a remote OM file over HTTP using range requests.
//!
//! Run with: `cargo run --example http_backend --features http-backend`
use omfiles::HttpBackend;
use omfiles::reader_async::OmFileReaderAsync;
use omfiles::traits::OmFileAsyncReadable;
use std::sync::Arc;

fn main() {
    smol::block_on(async {
        let url = "https://openmeteo.s3.amazonaws.com/data_spatial/dwd_icon/2026/09/19/0000Z/2026-09-19T1000.om";

        // Only issues a HEAD request to determine the file size.
        let backend = Arc::new(HttpBackend::new(url)
            .await.expect("failed to reach remote file"));

        // Only reads the trailer/header and metadata blocks, not the whole 170 MB file.
        let root = OmFileReaderAsync::new(backend)
            .await
            .expect("failed to open remote OM file");

        // Fetch the temperature_2m child.
        // In .om files, the LUT is indexed numerically, so we don't know which child is "temperature_2m" a priori.
        // Therefore, get_child_by_name must fetch (the metadata of) all children to find the requested child.
        // In .om files, there are usually 100 - 150 children, so this is limited by network latency.
        let temperature_reader = root
            .get_child_by_name("temperature_2m")
            .await
            .expect("variable not found");

        let temperature = temperature_reader
            .expect_array()
            .expect("not an array variable");

        // Reads only the compressed chunks that overlap this index range via HTTP range requests,
        // e.g. a small window instead of the full global grid.
        let subset: ndarray::ArrayD<f32> = temperature.read(&[0..4, 0..4]).await.unwrap();
        println!("{subset}");
    });
}
