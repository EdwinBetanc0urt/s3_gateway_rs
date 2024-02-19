use std::{env, io::Error, io::ErrorKind};
use std::str::FromStr;

use http::Method;
use minio::s3::args::{GetPresignedObjectUrlArgs, RemoveObjectArgs};
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use regex::Regex;

fn get_valid_path(_client_id: Option<String>, _container_id: Option<String>, _container_type: Option<String>, _table_name: Option<String>, _column_name: Option<String>, _record_id: Option<String>, _user_id: Option<String>, _include_user: bool) -> Result<String, std::io::Error> {
    if _client_id.to_owned().is_none() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Client ID is Mandatory"))
    }
    if _container_id.to_owned().is_none() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Container ID is Mandatory"))
    }
    if _container_type.to_owned().is_none() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Container Type is Mandatory"))
    }
    if _record_id.to_owned().is_some() && _table_name.to_owned().is_none() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Table Name is Mandatory"))
    }
    if _record_id.to_owned().is_none() && _table_name.to_owned().is_some() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Record ID is Mandatory"))
    }
    if _column_name.to_owned().is_some() && _table_name.to_owned().is_none() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Table Name is Mandatory"))
    }
    if !matches!(_container_type.clone().unwrap().as_ref(), "window" | "process" | "report" | "browser" | "form") {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Invalid Container Type"))
    }
    //  Client
    let mut _folder = get_valid_path_name(_client_id.to_owned().unwrap());
    _folder.push_str("/");
    //  Validate if is private access
    if _include_user && _user_id.is_some() {
        _folder.push_str(&get_valid_path_name(_user_id.unwrap()));
        _folder.push_str("/");    
    }
    //  Container
    //  Continer Type
    _folder.push_str(&get_valid_path_name(_container_type.unwrap()));
    _folder.push_str("-");
    _folder.push_str(&get_valid_path_name(_container_id.unwrap()));
    //  Table Name
    if _table_name.to_owned().is_some() {
        _folder.push_str("/");
        _folder.push_str(&get_valid_path_name(_table_name.unwrap()));
        _folder.push_str("/");
        _folder.push_str(&get_valid_path_name(_record_id.unwrap()));
    }
    //  Column
    if _column_name.to_owned().is_some() {
        _folder.push_str("/");
        _folder.push_str(&get_valid_path_name(_column_name.unwrap()));
    }
    let _final_folder = _folder.to_owned().to_lowercase();
    Ok(_folder.to_owned().to_lowercase())
}

fn get_valid_path_name(_value: String) -> String {
    let regex = Regex::new(r"[^A-Za-z0-9]").unwrap();
    regex.replace_all(&_value, "_").to_string()
}

fn get_valid_file_path(_value: String) -> String {
    let regex = Regex::new(r"[^A-Za-z0-9.]").unwrap();
    regex.replace_all(&_value, "").to_string()
}

pub fn get_valid_file_name(_client_id: Option<String>, _container_id: Option<String>, _file_name: Option<String>, _container_type: Option<String>, _table_name: Option<String>, _column_name: Option<String>, _record_id: Option<String>, _user_id: Option<String>) -> Result<String, std::io::Error> {
    if _file_name.to_owned().is_none() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "File Name is Mandatory"))
    }
    let _value = get_valid_path(_client_id, _container_id, _container_type, _table_name, _column_name, _record_id, _user_id, true);
    match _value {
        Ok(_folder_name) => {
            let mut _valid_file_name = _folder_name;
            _valid_file_name.push_str("/");
            _valid_file_name.push_str(&get_valid_file_path(_file_name.unwrap()));
            let _final_file_name = _valid_file_name.to_owned().to_lowercase();
            Ok(_final_file_name)
        },
        Err(error) => Err(Error::new(ErrorKind::InvalidData.into(), error))
    }
}

pub async fn request_signed_url(_file_name: String, _method: Method, _seconds: Option<u32>) -> Result<String, std::io::Error> {
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

    let static_provider: StaticProvider = StaticProvider::new(
        &_api_key, 
        &_secret_key, None);

    let client = Client::new(_base_url.clone(), Some(Box::new(static_provider)), None, None).unwrap();

    let args_to_match = GetPresignedObjectUrlArgs::new(
        &_bucket_name,
        _file_name.as_str(),
        _method,
    );

    match args_to_match {
        Ok(value) => {
            let mut _presigned_parameters = value;
            match _seconds {
                Some(seconds) => _presigned_parameters.expiry_seconds = Some(seconds),
                None => {

                }
            };
            match client.get_presigned_object_url(&_presigned_parameters).await {
                Ok(url) => Ok(url.url),
                Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error))
            }
        },
        Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error))
    }
}

pub async fn delete_object(_file_name: String) -> Result<(), std::io::Error> {
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

    let static_provider: StaticProvider = StaticProvider::new(
        &_api_key, 
        &_secret_key, None);

    let client = Client::new(_base_url.clone(), Some(Box::new(static_provider)), None, None).unwrap();

    let args_to_match = RemoveObjectArgs::new(
        &_bucket_name,
        _file_name.as_str()
    );

    match args_to_match {
        Ok(value) => {
            match client.remove_object(&value).await {
                Ok(_) => Ok(()),
                Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error))
            }
        },
        Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error))
    }
}