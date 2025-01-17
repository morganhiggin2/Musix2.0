use aws_config::{self, meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3;
use std::path::Path;

// Get the file from s3 and write into the desired location
pub async fn fetch_file_from_s3(s3_path: String, local_write_path: &Path) -> Result<(), String> {
    // Extract bucket and key from s3 path

    //TODO is it loading the aws config from the local place?
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-1");
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region_provider)
        .load()
        .await;

    let client = aws_sdk_s3::Client::from_conf(config);

    // TODO write file to local path
    let object = client
        .get_object()
        .bucket("my_bucket")
        .key("my_key")
        .send()
        .await?;

    let body = object.body().await?;

    // Write the file to the local path

    return Ok(());
}

// Write file into the desired s3 location from the desired location

/*
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{config::Region, Client};
*/
