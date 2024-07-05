use std::collections::HashMap;
use std::{env, io::Error, io::ErrorKind};
use std::str::FromStr;

use http::Method;
use minio::s3::args::{GetPresignedObjectUrlArgs, ListObjectsV2Args, RemoveObjectArgs};
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::response::ListObjectsV2Response;
use regex::Regex;
use serde::Serialize;
use std::path::Path;

fn get_valid_path(_client_id: Option<String>, _container_id: Option<String>, _container_type: Option<String>, _table_name: Option<String>, _column_name: Option<String>, _record_id: Option<String>, _user_id: Option<String>, _role_id: Option<String>, _include_access: bool) -> Result<String, std::io::Error> {
    if _client_id.to_owned().is_none() {
		log::error!("Client ID is Mandatory");
        return Err(Error::new(ErrorKind::InvalidData.into(), "Client ID is Mandatory"))
    }
    if _container_type.to_owned().is_none() {
        return Err(Error::new(ErrorKind::InvalidData.into(), "Container Type is Mandatory"))
		log::error!("Container Type is Mandatory");
    }

    if _record_id.to_owned().is_some() && _table_name.to_owned().is_none() {
		log::error!("Table Name is Mandatory");
		return Err(Error::new(ErrorKind::InvalidData.into(), "Table Name is Mandatory"))
    }
    if _record_id.to_owned().is_none() && _table_name.to_owned().is_some() {
		log::error!("Record ID is Mandatory");
        return Err(Error::new(ErrorKind::InvalidData.into(), "Record ID is Mandatory"))
    }
    if _column_name.to_owned().is_some() && _table_name.to_owned().is_none() {
		log::error!("Table Name is Mandatory");
        return Err(Error::new(ErrorKind::InvalidData.into(), "Table Name is Mandatory"))
    }
    if !matches!(_container_type.clone().unwrap().as_ref(), "window" | "process" | "report" | "browser" | "form" | "application" | "resource" | "attachment") {
		log::error!("Invalid Container Type");
        return Err(Error::new(ErrorKind::InvalidData.into(), "Invalid Container Type"))
    }
    if _container_id.is_none() && !_container_type.to_owned().unwrap().eq("attachment") {
		log::error!("Container ID is Mandatory");
		return Err(Error::new(ErrorKind::InvalidData.into(), "Container ID is Mandatory"))
	}
    if _table_name.is_none() || _record_id.is_none() {
        if _container_type.to_owned().unwrap().eq("attachment") {
			log::error!("Invalid Container Type (Mandatory Record ID and Table Name)");
            return Err(Error::new(ErrorKind::InvalidData.into(), "Invalid Container Type (Mandatory Record ID and Table Name)"))
        }
    }
    //  Client
    let mut _folder = get_valid_path_name(_client_id.to_owned().unwrap());
    _folder.push_str("/");
    //  Validate if is private access
    if _include_access && (_user_id.is_some() || _role_id.is_some()) {
        if _user_id.is_some() {
            _folder.push_str("user");
            _folder.push_str("/");
            _folder.push_str(&get_valid_path_name(_user_id.unwrap()));
        } else {
            _folder.push_str("role");
            _folder.push_str("/");
            _folder.push_str(&get_valid_path_name(_role_id.unwrap()));
        }
        _folder.push_str("/");
    } else {
        _folder.push_str("client");
        _folder.push_str("/");
    }

    //  Container Type
    _folder.push_str(&get_valid_path_name(_container_type.unwrap()));
	//  Container ID
	if _container_id.is_some() {
		_folder.push_str("/");
        _folder.push_str(&get_valid_path_name(_container_id.unwrap()));
	}
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
    let regex = Regex::new(r"[^A-Za-z0-9-]").unwrap();
    regex.replace_all(&_value, "_").to_string()
}

fn get_valid_file_path(_value: String) -> String {
	let regex = Regex::new(r"[^A-Za-z0-9.-_]").unwrap();
    regex.replace_all(&_value, "_").to_string()
}

pub fn get_valid_file_name(_client_id: Option<String>, _container_id: Option<String>, _file_name: Option<String>, _container_type: Option<String>, _table_name: Option<String>, _column_name: Option<String>, _record_id: Option<String>, _user_id: Option<String>, _role_id: Option<String>) -> Result<String, std::io::Error> {
    if _file_name.to_owned().is_none() {
		log::error!("File Name is Mandatory");
        return Err(Error::new(ErrorKind::InvalidData.into(), "File Name is Mandatory"))
    }
    let _value = get_valid_path(_client_id, _container_id, _container_type, _table_name, _column_name, _record_id, _user_id, _role_id, true);
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

pub async fn get_list_objects(_client_id: Option<String>, _container_id: Option<String>, _container_type: Option<String>, _table_name: Option<String>, _column_name: Option<String>, _record_id: Option<String>, _user_id: Option<String>, _role_id: Option<String>) -> Result<ResourceResponse, std::io::Error> {
    let _s3_url =  match env::var("S3_URL") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `S3_URL` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _bucket_name =  match env::var("BUCKET_NAME") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `BUCKET_NAME` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _api_key =  match env::var("API_KEY") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `API_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _secret_key =  match env::var("SECRET_KEY") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `SECRET_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _manage_https =  match env::var("MANAGE_HTTPS") {
        Ok(value) => {
            value.eq("Y")
        },
        Err(_) => {
            log::info!("Variable `MANAGE_HTTPS` Not found");
            false
        },
    };
    let _ssl_cert_file =  match env::var("SSL_CERT_FILE") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `SSL_CERT_FILE` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let mut _base_url: BaseUrl = match BaseUrl::from_str(&_s3_url) {
        Ok(url) => url,
        Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error)),
    };
    _base_url.https = _manage_https;
    let _cert_file_path= match _manage_https {
        true => Some(Path::new(&_ssl_cert_file)),
        false => None,
    };
    let static_provider: StaticProvider = StaticProvider::new(
        &_api_key, 
        &_secret_key, None);

    let client = Client::new(_base_url.clone(), Some(Box::new(static_provider)), _cert_file_path, None).unwrap();
    let _value = get_valid_path(_client_id, _container_id, _container_type, _table_name, _column_name, _record_id, _user_id, _role_id, true);
    let _prefix = match _value {
        Ok(_folder_name) => Some(_folder_name),
        Err(error) => {
            log::warn!("Error Getting path {:?}", error);
            return Err(Error::new(ErrorKind::InvalidData.into(), error))
        }
    };
    let mut _args = ListObjectsV2Args::new(&_bucket_name).unwrap();
    _args.prefix = _prefix.as_deref();
    match client
            .list_objects_v2(&_args)
            .await {
        Ok(value) => Ok(ResourceResponse::new(value)),
        Err(error) => Err(Error::new(ErrorKind::InvalidData.into(), error)),
    }
    
}

#[derive(Serialize, Debug, Clone)]
pub struct PresignedObject {
    pub url: Option<String>,
    pub file_name: Option<String>
}

#[derive(Serialize, Debug, Clone)]
pub struct ResourceResponse {
    pub parent_folder: Option<String>,
    pub resources: Option<Vec<Resource>>
}

#[derive(Serialize, Debug, Clone)]
pub struct Resource {
    pub name: String,
    pub last_modified: Option<String>,
    pub etag: Option<String>, // except DeleteMarker
    pub owner_name: Option<String>,
    pub size: Option<usize>, // except DeleteMarker
    pub storage_class: Option<String>,
    pub is_latest: bool,            // except ListObjects V1/V2
    pub version_id: Option<String>, // except ListObjects V1/V2
    pub user_metadata: Option<HashMap<String, String>>,
    pub is_prefix: bool,
    pub is_delete_marker: bool,
    pub encoding_type: Option<String>,
    pub content_type: Option<String>,
}

impl ResourceResponse {
    pub fn new(_data: ListObjectsV2Response) -> Self {
        ResourceResponse {
            parent_folder: _data.prefix,
            resources: Some(_data.contents.iter().map(|_content| {
                let _file_name = _content.to_owned().name;
                let _content_type = mime_guess::from_path(Path::new(&_file_name)).first_or_octet_stream();
                Resource { 
                    last_modified: match _content.last_modified {
                        Some(date) => Some(date.format("%Y-%m-%d %H:%M:%S").to_string()),
                        None => None
                    },
                    name: _file_name, 
                    etag: _content.to_owned().etag, 
                    owner_name: _content.to_owned().owner_name, 
                    size: _content.size, 
                    storage_class: _content.to_owned().storage_class, 
                    is_latest: _content.is_latest, 
                    version_id: _content.to_owned().version_id, 
                    user_metadata: _content.to_owned().user_metadata,
                    is_prefix: _content.is_prefix, 
                    is_delete_marker: _content.is_delete_marker, 
                    encoding_type: _content.to_owned().encoding_type,
                    content_type: Some(_content_type.to_string())
                }
            }).collect::<Vec<Resource>>()),
        }
    }
}


pub async fn request_signed_url(_file_name: String, _method: Method, _seconds: Option<u32>) -> Result<String, std::io::Error> {
    let _s3_url =  match env::var("S3_URL") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `S3_URL` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _bucket_name =  match env::var("BUCKET_NAME") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `BUCKET_NAME` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _api_key =  match env::var("API_KEY") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `API_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _secret_key =  match env::var("SECRET_KEY") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `SECRET_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _manage_https =  match env::var("MANAGE_HTTPS") {
        Ok(value) => {
            value.eq("Y")
        },
        Err(_) => {
            log::info!("Variable `MANAGE_HTTPS` Not found");
            false
        },
    };
    let _ssl_cert_file =  match env::var("SSL_CERT_FILE") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `SSL_CERT_FILE` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let mut _base_url: BaseUrl = match BaseUrl::from_str(&_s3_url) {
        Ok(url) => url,
        Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error)),
    };
    _base_url.https = _manage_https;
    let _cert_file_path= match _manage_https {
        true => Some(Path::new(&_ssl_cert_file)),
        false => None,
    };
    let static_provider: StaticProvider = StaticProvider::new(
        &_api_key, 
        &_secret_key, None);

    let client = Client::new(_base_url.clone(), Some(Box::new(static_provider)), _cert_file_path, None).unwrap();
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
			log::warn!("Variable `S3_URL` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _bucket_name =  match env::var("BUCKET_NAME") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `BUCKET_NAME` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _api_key =  match env::var("API_KEY") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `API_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _secret_key =  match env::var("SECRET_KEY") {
        Ok(value) => value,
        Err(_) => {
			log::warn!("Variable `SECRET_KEY` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let _manage_https =  match env::var("MANAGE_HTTPS") {
        Ok(value) => {
            value.eq("Y")
        },
        Err(_) => {
            log::info!("Variable `MANAGE_HTTPS` Not found");
            false
        },
    };
    let _ssl_cert_file =  match env::var("SSL_CERT_FILE") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `SSL_CERT_FILE` Not found");
            "".to_owned()
        }.to_owned(),
    };
    let mut _base_url: BaseUrl = match BaseUrl::from_str(&_s3_url) {
        Ok(url) => url,
        Err(error) => return Err(Error::new(ErrorKind::InvalidData.into(), error)),
    };
    _base_url.https = _manage_https;
    let _cert_file_path= match _manage_https {
        true => Some(Path::new(&_ssl_cert_file)),
        false => None,
    };
    let static_provider: StaticProvider = StaticProvider::new(
        &_api_key, 
        &_secret_key, None);

    let client = Client::new(_base_url.clone(), Some(Box::new(static_provider)), _cert_file_path, None).unwrap();

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
