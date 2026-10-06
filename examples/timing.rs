use std::sync::Arc;
use std::time::Instant;

use futures_lite::future::block_on;
use ndarray::ArrayD;
use omfiles::HttpBackend;
use omfiles::reader_async::OmFileReaderAsync;
use omfiles::traits::{OmArrayVariable, OmFileAsyncReadable};

fn main() {
    block_on(async {
    let om_url = "https://openmeteo.s3.amazonaws.com/data_spatial/dwd_icon/2026/09/22/0000Z/2026-09-22T1200.om?variable=temperature_2m";

    let now = Instant::now();
    // Issue a HEAD request to determine file size:
    let backend = Arc::new(HttpBackend::new(om_url)
      .await.expect("Failed to fetch remote data file"));
    println!("HEAD took {} ms", now.elapsed().as_millis());

    let now = Instant::now();
    // Read the header, trailer and root object:
    let root = OmFileReaderAsync::new(backend)
      .await.expect("Failed to read root object");
    println!("ROOT took {} ms", now.elapsed().as_millis());

    let now = Instant::now();
    // Locate the temperature object:
    let temp2m_var = root.get_child_by_name("temperature_2m")
      .await.expect("Failed to read temperature object");
    println!("GET_CHILD took {} ms", now.elapsed().as_millis());

    let now = Instant::now();
    // Convert it to array:
    let temp2m_arr = temp2m_var.expect_array().expect("Failed to convert to temperature array");

    let dimen = temp2m_arr.get_dimensions();
    assert!(dimen.len() == 2, "Temperature shape is not 2D");
    println!("ASSERT took {} ms", now.elapsed().as_millis());

    // read() took 33 secs. (of which alloc is 0)

    let now = Instant::now();
    // Read the entire array (for now):
    let area = [0..dimen[0], 0..dimen[1]];
    let mut out = ArrayD::<f32>::zeros(vec![dimen[0] as usize, dimen[1] as usize]);
    println!("ALLOC took {} ms", now.elapsed().as_millis());

    // Allocate the entire array (for now):
    let now = Instant::now();
    temp2m_arr.read_into::<f32>(
            &mut out,
            &area,
            &vec![0; 2],
            &dimen).await.expect("Failed to read temperatures");
    println!("READ_INTO took {} ms", now.elapsed().as_millis());
    })
}
