use omfiles::HttpBackend;
use omfiles::reader_async::OmFileReaderAsync;
use omfiles::traits::{OmArrayVariable, OmFileAsyncReadable, OmFileVariable};
use std::sync::Arc;

fn main() {
    smol::block_on(async {
        let url = "https://openmeteo.s3.amazonaws.com/data_spatial/dwd_icon/2026/09/19/0000Z/2026-09-19T1000.om";
        let backend = HttpBackend::new(url).await.expect("failed to create backend");
        println!(
            "file size: {}",
            omfiles::traits::OmFileReaderBackendAsync::count_async(&backend)
        );
        let reader = OmFileReaderAsync::new(Arc::new(backend)).await.expect("failed to open reader");
        println!("name: {}", reader.name());
        println!("children: {}", reader.number_of_children());

        for i in 0..reader.number_of_children().min(10) {
            let child = reader.get_child_by_index(i).await.unwrap();
            println!("  child[{i}]: {}", child.name());
        }

        let temp = reader
            .get_child_by_name("temperature_2m")
            .await
            .expect("temperature_2m variable not found");
        let array = temp.expect_array().unwrap();
        println!("temperature_2m dims: {:?}", array.get_dimensions());

        // Read a small 4x4 window (partial read via HTTP range requests)
        let data: ndarray::ArrayD<f32> = array.read(&[0..4, 0..4]).await.unwrap();
        println!("partial read result:\n{data}");
    });
}
