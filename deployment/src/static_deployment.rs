use std::path::Path;
use anyhow::Result;
use aws_sdk_s3::model::{Delete, Object, ObjectIdentifier};
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;

use futures::FutureExt;
use tokio::fs::File;
use tokio::process::Command;

use walkdir::WalkDir;
use crate::StreamExt;


async fn list_old_assets(client: &Client, cf_prefix: &str, yew_crate_name: &str) -> Vec<String> {

    let objects = client
        .list_objects_v2()
        .bucket("siyuanyan")
        .prefix(format!("website-assets/{cf_prefix}"))
        .send()
        .await
        .unwrap();

    let keys = objects.contents.unwrap_or_default()
        .into_iter()
        .filter_map(|Object{key,..}|{
            key
        })
        .map(
            |k| (k, client.clone())
        );

    let keys = futures::stream::iter(keys);

    keys.filter_map(|(k,c)|async move {

        let params = c.head_object()
            .bucket("siyuanyan")
            .key(&k)
            .send().await.unwrap();
        if let Some(remote_crate_name) = params.metadata.unwrap_or_default().get("yew-crate"){
            if remote_crate_name == yew_crate_name {
                Some(k)
            }else{
                None
            }
        }else{
            None
        }
    })


        .collect()
        .await




}

/// `cf_prefix`: no starting slash, no ending slash
pub async fn deploy<P: AsRef<Path>>(cf_prefix: &str, yew_crate: P) -> Result<()> {
    dotenv::dotenv().ok();

    let yew_crate_name = yew_crate.as_ref().file_name().unwrap().to_string_lossy();

    let shared_config = aws_config::from_env().region("ap-east-1").load().await;

    let client = Client::new(&shared_config);

    let trunk = Command::new("trunk")
        .arg("--config")
        .arg("Trunk-release.toml")
        .arg("build")
        .arg("--release")
        .current_dir(yew_crate.as_ref())
        .output()
        .then(|x| async {
            let x = x.expect("failed to call trunk build");
            println!("\tfinished trunk build");
            x
        });

    let compression = crate::gzip_release_dir(yew_crate.as_ref().join("dist/release/uncompressed"));

    println!(
        "building distribution and listing old assets from the bucket"
    );
    let (trunk_build_output, old_objects) = tokio::join!(
        trunk,
        list_old_assets(&client, cf_prefix, &yew_crate_name).then(|x| async {
            println!("\tfinished listing old objects");
            x
        }),

    );

    if !trunk_build_output.status.success() {
        eprintln!("trunk build returned nonzero exit code, here's the stderr:");
        eprint!("{}", String::from_utf8(trunk_build_output.stderr).unwrap());
        anyhow::bail!("terminated");
    }

    println!("\t\tfound {} old objects", old_objects.len());

    let old_objects: Vec<_> = old_objects
        .into_iter()
        .map(|x| {
            println!("\t\t\t{x}");
            ObjectIdentifier::builder().set_key(Some(x)).build()
        })
        .collect();

    if !old_objects.is_empty() {
        println!("deleting old objects...");
        client
            .delete_objects()
            .bucket("siyuanyan")
            .delete(Delete::builder().set_objects(Some(old_objects)).build())
            .send()
            .await?;
        println!("\told objects deleted");
    }

    let upload_to_s3 = async {
        let mut output = "".to_string();
        for entry in WalkDir::new(yew_crate.as_ref().join("dist/release/brotli"))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy();
            if path.is_file() && file_name != "index.html" {
                output.push_str("\tuploaded ");
                output.push_str(&file_name);
                output.push('\n');

                let body = ByteStream::from_file(File::open(path).await.unwrap())
                    .await
                    .unwrap();

                let put = client
                    .put_object()
                    .bucket("siyuanyan")
                    .key(format!(
                        "website-assets/{cf_prefix}/{file_name}"
                    ))
                    .metadata("yew-crate", yew_crate_name.clone())
                    .body(body);
                let put = if file_name.ends_with(".js") {
                    put.content_type("text/javascript")
                        .content_encoding("br")
                } else if file_name.ends_with(".wasm") {
                    put.content_type("application/wasm")
                        .content_encoding("br")
                } else {
                    put
                };
                put.send().await.expect("failed to upload file");
            }
        }
        output
    };

    let scp = Command::new("scp")
        .arg(yew_crate.as_ref().join("dist/release/uncompressed/index.html"))
        .arg(format!("root@xray:/site/{cf_prefix}/index.html"))
        .output();

    let remove = tokio::fs::remove_dir_all(yew_crate.as_ref().join("dist/release/brotli"));

    println!("compression and uploading objects to S3");
    println!("uploading index.html to the backend");
    let (s3, scp) = tokio::join!(

        remove
        .then(|_| compression)
        .then(|r| async move {
            print!("{r}");
            upload_to_s3.await
        })

        , scp);

    print!("{s3}");

    let scp = scp.expect("failed to call scp");
    if !scp.status.success() {
        eprintln!("scp exited with nonzero code, here's the stderr");
        eprint!("{}", String::from_utf8(scp.stderr).unwrap());
        anyhow::bail!("terminated");
    }
    println!("\tuploaded index.html");

    Ok(())
}
