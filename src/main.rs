use azure_core::prelude::*;
use azure_storage::blob::prelude::*;
use azure_storage::core::prelude::*;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::error::Error;
use std::fmt;
use clap::{App, Arg, ArgGroup};
use anyhow::{Result, anyhow};
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
struct Configs {
    storage_account: String,
    storage_master_key: String,
    local: String,
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Parse command line arguments
    let args = App::new("azure-storage")
        // headers
        .version(env!("CARGO_PKG_VERSION"))
        .author("ADVALY SYSTEM Inc.")
        .about("Azure Storage file uploader and downloader")

        // mode
        .arg(Arg::with_name("list").help("List objects on remote"))
        .arg(Arg::with_name("get").help("Get a blob from remote"))
        .arg(Arg::with_name("put").help("Put a block blob to remote"))
        .arg(Arg::with_name("append").help("Append a file to existing append blob"))
        .arg(Arg::with_name("put-append").help("Create a new append blob to remote"))
        .arg(Arg::with_name("delete").help("Delete a blob from remote"))
        .group(ArgGroup::with_name("mode")
            .args(&["list", "get", "put", "append", "put-append", "delete"])
            .required(true)
        )

        // options
        .arg(Arg::with_name("local")
           .short("l").long("local")
            .help("Local file path to put or get")
            .takes_value(true)
        )
        .arg(Arg::with_name("container")
           .short("c").long("container")
            .help("Remote container name on Azure Storage")
            .takes_value(true)
        )
        .arg(Arg::with_name("blob")
           .short("b").long("blob")
            .help("Remote blob name on Azure Storage")
            .takes_value(true)
        )
        .arg(Arg::with_name("storage account")
            .short("a").long("storage_account")
            .help("STORAGE_ACCOUNT")
            .takes_value(true)
        )
        .arg(Arg::with_name("storage master key")
            .short("k").long("storage_master_key")
            .help("STORAGE_MASTER_KEY")
            .takes_value(true)
        )
        .arg(Arg::with_name("config")
            .long("config")
            .help("Config file path")
            .takes_value(true)
            .default_value("azure-storage.json")
        )
        .arg(Arg::with_name("debug")
            .long("debug")
            .help("Enable debug print")
        )
        .get_matches();

    // Read config parameters if exist
    let mut cfg: Configs = match File::open(args.value_of("config").unwrap()) {
        Ok(file) => serde_json::from_reader(BufReader::new(file))?,
        Err(_) => Default::default()
    };

    // Overwrite config parameters by command line options
    args.value_of("storage account").map(|v| cfg.storage_account = v.into());
    args.value_of("storage master key").map(|v| cfg.storage_master_key = v.into());
    args.value_of("local").map(|v| cfg.local = v.into());

    // debug print
    if args.is_present("debug") {
        println!("{:#?}", cfg);
    }

    // Get storage keys from environment variable if no config parameter
    let account = match cfg.storage_account.as_str() {
        "" => std::env::var("STORAGE_ACCOUNT").expect("STORAGE_ACCOUNT is not defined"),
        _ => cfg.storage_account
    };

    let master_key = match cfg.storage_master_key.as_str() {
        "" => std::env::var("STORAGE_MASTER_KEY").expect("STORAGE_MASTER_KEY is not defined"),
        _ => cfg.storage_master_key
    };

    // Create a storage client object
    let http_client = new_http_client();
    let storage_client =
        StorageAccountClient::new_access_key(http_client, &account, &master_key).as_storage_client();

    // Perform Azure Storage access
    let local = if cfg.local != "" { Some(cfg.local.as_str()) } else { None };
    azure_storage(storage_client, 
        args.value_of("mode"), 
        args.value_of("container"), 
        args.value_of("blob"), 
        local, 
        args.is_present("debug"))?;

    Ok(())
}

#[tokio::main]
async fn azure_storage(storage_client: Arc<StorageClient>, mode: Option<&str>, container: Option<&str>, blob: Option<&str>, local: Option<&str>, debug: bool)
    -> Result<(), Box<dyn Error + Send + Sync>>
{
    if debug {
        println!("mode = {:?}", mode);
        println!("container name = {:?}", container);
        println!("blob name = {:?}", blob);
        println!("local path = {:?}", local);
        println!("\n{:#?}", storage_client);
    }

    match mode {
        // List remote objects
        Some("list") | None => {
            // blobs (if specified container name)
            if let Some(container) = container {
                let res = storage_client
                    .as_container_client(container)
                    .list_blobs()
                    .execute()
                    .await?;

                println!("List of {} blobs in container '{}'", res.blobs.blobs.len(), container);
                for blob in res.blobs.blobs.iter() {
                    println!(" {} {:>8} {:>10} {}",
                        blob.properties.last_modified,
                        blob.properties.content_length,
                        blob.properties.blob_type.to_string(),
                        blob.name);
                }
                debug_print(res, debug);
            }

            // containers (if no container name specified)
            else {
                let res = storage_client
                    .list_containers()
                    .execute()
                    .await?;

                println!("List of {} containers", res.incomplete_vector.len());
                for container in res.incomplete_vector.iter() {
                    println!(" {} {}", container.last_modified, container.name);
                }
                debug_print(res, debug);
            }
        },

        // Create a just new append blob to remote. no local file required
        Some("put-append") => {
            let container = container.ok_or(anyhow!("No container name specified"))?;
            let blob = blob.ok_or(anyhow!("No blob name specified"))?;

            // Create a blob instance
            let blob_client = storage_client
                .as_container_client(container)
                .as_blob_client(blob);

            let res = blob_client
                .put_append_blob()
                .execute()
                .await?;

            debug_print(res, debug);
        },

        // Put or append a file to remote
        Some("put" | "append") => {
            // Check path
            let local_path = local.ok_or(anyhow!("No local path specified"))?;
            let container = container.ok_or(anyhow!("No container name specified"))?;

            // Check local_path. Use the local filename as blob name if no blob name is specified.
            let blob = match blob {
                Some(v) => v,
                None => Path::new(local_path).file_name()
                    .ok_or(anyhow!("Cannot extract filename from local path"))?.to_str().unwrap()
            };
            
            // Create a blob instance
            let blob_client = storage_client
                .as_container_client(container)
                .as_blob_client(blob);
    
            // Read data from file
            let mut buffer = Vec::new();
            File::open(local_path).and_then(|mut f| f.read_to_end(&mut buffer))?;

            // this is not mandatory but it helps preventing spurious data to be uploaded
            let hash = md5::compute(&buffer).into();

            // [put] Put to remote
            if mode.unwrap() == "put" {
                let res = blob_client
                    .put_block_blob(buffer)
                    .hash(&hash)
                    .execute()
                    .await?;
                debug_print(res, debug);
            }

            // [append] Append to remote blob
            else {
                let res = blob_client
                    .append_block(buffer)
                    .hash(&hash)
                    .execute()
                    .await?;
                debug_print(res, debug);
            }
        },

        // Get a file from remote
        Some("get") => {
            // Check remote path
            let container = container.ok_or(anyhow!("No container name specified"))?;
            let blob = blob.ok_or(anyhow!("No blob name specified"))?;

            // Check local_path. Add the blob name as local filename if local path is directory.
            let local_path = local.ok_or(anyhow!("No local path specified"))
                .map(|v| {
                    let mut path = PathBuf::from(v);
                    if path.exists() && path.is_dir() {
                        path = path.join(blob);
                        if debug {
                            println!("local path (complemented) = {:?}", path);
                        }
                    }
                    path
                })?;
            
            // Create a blob instance
            let blob_client = storage_client
                .as_container_client(container)
                .as_blob_client(blob);
    
            // Get the remote file
            let res = blob_client
                .get()
                .execute()
                .await?;

            // Write to a file
            File::create(local_path).and_then(|mut f| f.write_all(&res.data))?;

            debug_print(res, debug);
        },

        // Delete a blob from remote
        Some("delete") => {
            // Check remote path
            let container = container.ok_or(anyhow!("No container name specified"))?;
            let blob = blob.ok_or(anyhow!("No blob name specified"))?;

            // Create a blob instance
            let blob_client = storage_client
                .as_container_client(container)
                .as_blob_client(blob);

            // Delete a blob
            let res = blob_client
                .delete()
                .execute()
                .await?;

            debug_print(res, debug);
        },

        // Error
        Some(_) => {
            return Err(anyhow!("Invalid mode").into())
        }
    }

    Ok(())
}

fn debug_print<T>(obj: T, debug: bool) where T: fmt::Debug
{
    if debug {
        println!("\n{:#?}", obj);
    }
}