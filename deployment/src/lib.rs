pub mod static_deployment;

use std::path::Path;
use std::time::Instant;
use async_compression::tokio::write::BrotliEncoder;
use futures::StreamExt;
use walkdir::WalkDir;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter, AsyncReadExt};

/// gzip everything except index.html
async fn gzip_release_dir<P: AsRef<Path>>(dir: P) -> String{

   let compress = WalkDir::new(dir)
        .into_iter()

        .filter_map(|dir| dir.ok())
        .filter_map(|d|{
            let path = d.path().to_owned();
            (path.is_file() && path.file_name().unwrap().to_string_lossy() != "index.html" ).then(||{
                path
                // path.file_name().unwrap().to_string_lossy()
            })
        })
        .map(|path| {

            let dir = path.parent().unwrap().parent().unwrap().join("brotli");
            std::fs::create_dir_all(&dir).unwrap();

            tokio::task::spawn(async move {


                let mut input = vec![];
                File::open(path.as_path()).await.unwrap().read_to_end(& mut input ).await.ok();



                let output = BufWriter::new(File::create(dir.join(path.file_name().unwrap())).await.unwrap());
                let mut encoder = BrotliEncoder::new(output);
                let start = Instant::now();

                encoder.write_all(input.as_slice()).await.ok();

                encoder.shutdown().await.unwrap();


                let mut contents = encoder.into_inner();
                contents.flush().await.unwrap();
                format!(
                    "\t{} compressed, source len: {:?}\ttarget len: {:?}\telapsed: {:?}\n",
                    path.file_name().unwrap().to_string_lossy(), input.len(),contents.into_inner().metadata().await.unwrap().len(),start.elapsed()
                )

            })
        });

    let compress = futures::stream::iter(compress);
    compress.fold("".to_string(), |mut acc, new|async move {
        let new = new.await.unwrap();
        acc.push_str(&new);
        acc
    }).await
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compress_fov_works() {
        gzip_release_dir("../fov-calculator/dist/release/uncompressed").await;

    }
}