use std::{env, io::Error, io::ErrorKind};
use std::str::FromStr;

use minio::s3::args::GetPresignedObjectUrlArgs;
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;

pub async fn create_device_user(_file_name: String) -> Result<String, std::io::Error> {
    let _s3_url =  match env::var("S3_URL") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `S3_URL` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _bucket_name =  match env::var("BUCKET_NAME") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `BUCKET_NAME` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _api_key =  match env::var("API_KEY") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `API_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _secret_key =  match env::var("SECRET_KEY") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `SECRET_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let mut _base_url: BaseUrl = BaseUrl::from_str(&_s3_url).expect("Error with url");
    _base_url.https = false;

    let static_provider = StaticProvider::new(
        &_api_key, 
        &_secret_key, None);

        let client = Client::new(_base_url.clone(), Some(Box::new(static_provider)), None, None).unwrap();

    let args_to_match = GetPresignedObjectUrlArgs::new(
        &_bucket_name,
        _file_name.as_str(),
            http::Method::GET,
    );

    match args_to_match {
        Ok(value) => {
            match client.get_presigned_object_url(&value).await {
                Ok(url) => Ok(url.url),
                Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error))
            }
        },
        Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error))
    }
}