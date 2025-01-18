use aws_config::{self, meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3;
use std::{io::Write, path::Path};

// Get the object from s3 and write into the desired location
pub async fn write_s3_object_to_file(
    s3_uri: String,
    local_write_path: &Path,
) -> Result<(), String> {
    // Extract bucket and key from s3 uri
    // the uri will be of the following format: s3:bucket/key
    let mut s3_uri_split = s3_uri.splitn(1, '/');

    let bucket_name = match s3_uri_split.next() {
        Some(bucket_name) => bucket_name,
        None => {
            return Err(format!(
                "Could not extract bucket name from malformed s3 uri: {}",
                s3_uri
            ));
        }
    };

    let key_name = match s3_uri_split.next() {
        Some(bucket_name) => bucket_name,
        None => {
            return Err(format!(
                "Could not extract key name from malformed s3 uri: {}",
                s3_uri
            ));
        }
    };

    let region_provider = RegionProviderChain::default_provider().or_else("us-west-1");
    // This will load the configuration profile for the client from ~/.aws/config
    let credentials_provider =
        aws_config::profile::ProfileFileCredentialsProvider::builder().build();

    // create the aws config
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region_provider)
        .credentials_provider(credentials_provider)
        .load()
        .await;

    // create the s3 config
    let sdk_config = aws_sdk_s3::Config::new(&config);
    let client = aws_sdk_s3::Client::from_conf(sdk_config);

    // get the s3 object, load it into memory
    let mut s3_object = match client
        .get_object()
        .bucket(bucket_name)
        .key(key_name)
        .send()
        .await
    {
        Ok(s3_object) => s3_object,
        Err(e) => {
            return Err(format!("Error fetching file from s3: {}", e));
        }
    };

    // Create file
    match std::fs::create_dir_all(local_write_path) {
        Ok(()) => (),
        Err(e) => return Err(format!("Error creating local directory: {}", e)),
    };
    let mut file = match std::fs::File::create(local_write_path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Error creating file: {}", e)),
    };

    // Stream the contents of the object ot the file
    while let Some(bytes) = s3_object
        .body
        .try_next()
        .await
        .map_err(|err| format!("Failed to read from S3 download stream: {err:?}"))?
    {
        file.write_all(&bytes).map_err(|err| {
            format!("Failed to write from S3 download stream to local file: {err:?}")
        });
    }

    return Ok(());
}

// Write file into the desired s3 location from the desired location
pub async fn put_file_to_s3_object(s3_uri: String, local_read_path: &Path) -> Result<(), String> {
    // Extract bucket and key from s3 uri
    // the uri will be of the following format: s3:bucket/key
    let mut s3_uri_split = s3_uri.splitn(1, '/');

    let bucket_name = match s3_uri_split.next() {
        Some(bucket_name) => bucket_name,
        None => {
            return Err(format!(
                "Could not extract bucket name from malformed s3 uri: {}",
                s3_uri
            ));
        }
    };

    let key_name = match s3_uri_split.next() {
        Some(bucket_name) => bucket_name,
        None => {
            return Err(format!(
                "Could not extract key name from malformed s3 uri: {}",
                s3_uri
            ));
        }
    };

    let region_provider = RegionProviderChain::default_provider().or_else("us-west-1");
    // This will load the configuration profile for the client from ~/.aws/config
    let credentials_provider =
        aws_config::profile::ProfileFileCredentialsProvider::builder().build();

    // create the aws config
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region_provider)
        .credentials_provider(credentials_provider)
        .load()
        .await;

    // create the s3 config
    let sdk_config = aws_sdk_s3::Config::new(&config);
    let client = aws_sdk_s3::Client::from_conf(sdk_config);

    // Get the body of the file
    let file_byte_stream =
        match aws_sdk_s3::primitives::ByteStream::from_path(local_read_path).await {
            Ok(stream) => stream,
            Err(err) => return Err(format!("Error creating ByteStream from file: {}", err)),
        };

    // get the s3 object, load it into memory
    let _s3_object = match client
        .put_object()
        .bucket(bucket_name)
        .key(key_name)
        .body(file_byte_stream)
        .send()
        .await
    {
        Ok(s3_object) => s3_object,
        Err(e) => {
            return Err(format!("Error fetching file from s3: {}", e));
        }
    };

    return Ok(());
}
