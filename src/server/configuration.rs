use std::collections::BTreeSet;
use tokio::fs;
use std::path::{Path, PathBuf};
use dirs::home_dir;
use log::{debug, info, Level};
use reqwest::Url;
use thiserror::Error;

type ConfigurationResult<T> = Result<T,ConfigurationError>;

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("file system error")]
    FileSystemError(#[from] std::io::Error),
    #[error("this user home directory not found")]
    HomeDirectoryNotFound,
}


const BASE_FILTERS_URL: &str = "https://filters.privaxy.net";
const METADATA_FILE_NAME: &str = "metadata.json";
const CONFIGURATION_DIRECTORY_NAME: &str = ".privaxy";
const CONFIGURATION_FILE_NAME: &str = "config";
//
// pub struct Ca {
//     ca_certificate: String,
//     ca_private_key: String,
// }
//
// enum FilterGroup {
//     Default,
//     Regional,
//     Ads,
//     Privacy,
//     Malware,
//     Social,
// }
// pub struct Filter {
//     enabled_by_default: bool,
//     title: String,
//     group: FilterGroup,
//     file_name: String,
// }
#[derive(Debug)]
pub struct Configuration {
    pub exclusions: BTreeSet<String>,
    // pub custom_filter: Vec<String>,
    // ca: Ca,
    // pub filters: Vec<Filter>,
}

impl Configuration {
    pub async fn read_from_home(client: reqwest::Client) -> ConfigurationResult<Self>{
        let home_dir = get_home_directory()?;
        let configuration_directory = home_dir.join(CONFIGURATION_DIRECTORY_NAME).join("d");
        let configuration_file_path = configuration_directory.join(CONFIGURATION_FILE_NAME);
        Self::create_dir_if_missing(&configuration_directory, client).await?;
        Ok(
            Self { exclusions: BTreeSet::new() }
        )
    }

    async fn create_dir_if_missing(dir: &PathBuf, client: reqwest::Client) -> ConfigurationResult<()>{
        if let Err(err) = fs::metadata(dir).await {
            if err.kind() == std::io::ErrorKind::NotFound {
                debug!("Configuration directory not found, creating one");
                fs::create_dir(dir).await.unwrap();
            }
                let config = Self::new_default(client).await?;
        }
        Ok(())
    }

    async fn new_default(client: reqwest::Client) -> ConfigurationResult<Self>{
        let default_filter = Self::get_default_filters(client).await?;
        Ok(Configuration {exclusions: BTreeSet::new()})
    }

    async fn get_default_filters(client: reqwest::Client) -> ConfigurationResult<()> {
        let url = BASE_FILTERS_URL.parse::<Url>().unwrap();
        let filters_url = url.join(METADATA_FILE_NAME).unwrap();
        println!("{}", filters_url);

        let response = client.get(filters_url.as_str()).send().await.unwrap();

        Ok(())
    }
}


fn get_home_directory() -> ConfigurationResult<PathBuf> {
    match home_dir() {
        Some(dir) => Ok(dir),
        None => Err(ConfigurationError::HomeDirectoryNotFound),
    }
}