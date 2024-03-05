use std::env;
use dotenv::dotenv;
use s3_gateway_rs::controller::s3::{delete_object, get_list_objects, get_valid_file_name, request_signed_url, PresignedObject};
use salvo::{prelude::*, cors::Cors, http::header, hyper::Method};
extern crate serde_json;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
    dotenv().ok();
    SimpleLogger::new().env().init().unwrap();
    let host =  match env::var("HOST") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `HOST` Not found from enviroment, loaded from local IP");
            "127.0.0.1:7878".to_owned()
        }.to_owned(),
    };
    // TODO: Add support to allow requests from multiple origin
    let allowed_origin = match env::var("ALLOWED_ORIGIN") {
        Ok(value) => value,
        Err(_) => {
            log::info!("Variable `ALLOWED_ORIGIN` Not found from enviroment");
            "*".to_owned()
        }.to_owned(),
    };
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
    log::info!("Server Address: {:?}", host.clone());

    let cors_handler = Cors::new()
        .allow_origin(&allowed_origin.to_owned())
        .allow_methods(vec![Method::OPTIONS, Method::GET, Method::DELETE])
        .allow_headers(vec![header::ACCESS_CONTROL_REQUEST_METHOD, header::ACCESS_CONTROL_REQUEST_HEADERS, header::AUTHORIZATION])
        .into_handler()
    ;
    // TODO: Add generic ok response to OPTIONS http method
    let router = Router::new()
        .hoop(cors_handler)
        .push(
            Router::with_path("api")
                .push(
                    Router::with_path("resources")
                        .options(get_resources_file_container_based)
                        .get(get_resources_file_container_based)
                        .push(
                            Router::with_path("<**file_name>")
                                .options(get_resource)
                                .get(get_resource)
                                .delete(delete_resource)
                        )
                )
                .push(
                    Router::with_path("download-url/<**file_name>")
                        .options(get_presigned_url_download_file)
                        .get(get_presigned_url_download_file)
                )
                .push(
                    Router::with_path("presigned-url/<client_id>/<container_id>/<file_name>")
                        .options(get_presigned_url_put_file_container_based)
                        .get(get_presigned_url_put_file_container_based)
                )
        )
    ;
    log::info!("{:#?}", router);
    let acceptor = TcpListener::new(&host).bind().await;
    Server::new(acceptor).serve(router).await;
}

#[handler]
async fn get_resource<'a>(_req: &mut Request, _res: &mut Response) {
    let _file_name = _req.param::<String>("**file_name");
    let _seconds = _req.query::<u32>("seconds");
    if _file_name.is_some() {
        match request_signed_url(_file_name.unwrap(), http::Method::GET, _seconds).await {
            Ok(url) => _res.render(Redirect::permanent(url)),
            Err(error) => {
                _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                _res.render(Json(error.to_string()));
            }
        }
    } else {
        _res.render("File Name is mandatory".to_string());
        _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[handler]
async fn delete_resource<'a>(_req: &mut Request, _res: &mut Response) {
    let _file_name = _req.param::<String>("**file_name");
    if _file_name.is_some() {
        match delete_object(_file_name.unwrap()).await {
            Ok(_) => {
                
            },
            Err(error) => {
                _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                _res.render(Json(error.to_string()));
            }
        }
    } else {
        _res.render("File Name is mandatory".to_string());
        _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[handler]
async fn get_resources_file_container_based<'a>(_req: &mut Request, _res: &mut Response) {
    let _client_id = _req.query::<String>("client_id");
    let _container_id = _req.query::<String>("container_id");
    let _file_name = _req.param::<String>("file_name");
    let _container_type = _req.query::<String>("container_type");
    let _table_name = _req.query::<String>("table_name");
    let _column_name = _req.query::<String>("column_name");
    let _record_id = _req.query::<String>("record_id");
    let _user_id = _req.query::<String>("user_id");
    let _role_id = _req.query::<String>("role_id");
    let _seconds = _req.query::<u32>("seconds");
    let _object_list = get_list_objects(_client_id.to_owned(), _container_id.to_owned(), _container_type.to_owned(), _table_name.to_owned(), _column_name.to_owned(), _record_id.to_owned(), _user_id.to_owned(), _role_id.to_owned()).await;
    match _object_list {
        Ok(_objects) => {
           _res.render(Json(_objects))
        },
        Err(error) => {
            _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            _res.render(Json(error.to_string()));
        }
    }
}

#[handler]
async fn get_presigned_url_put_file_container_based<'a>(_req: &mut Request, _res: &mut Response) {
    let _client_id = _req.param::<String>("client_id");
    let _container_id = _req.param::<String>("container_id");
    let _file_name = _req.param::<String>("file_name");
    let _container_type = _req.query::<String>("container_type");
    let _table_name = _req.query::<String>("table_name");
    let _column_name = _req.query::<String>("column_name");
    let _record_id = _req.query::<String>("record_id");
    let _user_id = _req.query::<String>("user_id");
    let _role_id = _req.query::<String>("role_id");
    let _seconds = _req.query::<u32>("seconds");
    let _ = get_list_objects(_client_id.to_owned(), _container_id.to_owned(), _container_type.to_owned(), _table_name.to_owned(), _column_name.to_owned(), _record_id.to_owned(), _user_id.to_owned(), _role_id.to_owned()).await;
    //  Get Valid File Name
    let _file_name_to_store = get_valid_file_name(_client_id, _container_id, _file_name, _container_type, _table_name, _column_name, _record_id, _user_id, _role_id);
    match _file_name_to_store {
        Ok(_valid_file_name) => {
            match request_signed_url(_valid_file_name.to_owned(), http::Method::PUT, _seconds).await {
                Ok(url) => _res.render(Json(PresignedObject {
                    url: Some(url),
                    file_name: Some(_valid_file_name)
                })),
                Err(error) => {
                    _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    _res.render(Json(error.to_string()));
                }
            }
        },
        Err(error) => {
            _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            _res.render(Json(error.to_string()));
        }
    }
}

#[handler]
async fn get_presigned_url_download_file<'a>(_req: &mut Request, _res: &mut Response) {
    let _file_name = _req.param::<String>("**file_name");
    let _seconds = _req.query::<u32>("seconds");
    if _file_name.is_some() {
        match request_signed_url(_file_name.unwrap(), http::Method::GET, _seconds).await {
            Ok(url) => _res.render(Json(url)),
            Err(error) => {
                _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                _res.render(Json(error.to_string()));
            }
        }
    } else {
        _res.render("File Name is mandatory".to_string());
        _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
}
