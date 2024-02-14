use std::env;
use dotenv::dotenv;
use s3_gateway_rs::controller::s3::{delete_object, request_signed_url};
use salvo::{prelude::*, cors::Cors, hyper::Method};
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
    let allowed_origin =  match env::var("ALLOWED_ORIGIN") {
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
    .allow_methods(vec![Method::OPTIONS, Method::GET, Method::DELETE]).into_handler();
    let router = Router::new()
        .hoop(cors_handler)
        .push(
            Router::with_path("api/resources/<**file_name>")
                .get(get_resource)
                .delete(delete_file)
        )
        .push(
            Router::with_path("api/presigned-url/<**file_name>")
                .get(get_presigned_url_put_file)
        )
        .push(
            Router::with_path("api/download-url/<**file_name>")
                .get(get_presigned_url_download_file)
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
    println!("Epale: {:?}", _file_name);
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
async fn delete_file<'a>(_req: &mut Request, _res: &mut Response) {
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
async fn get_presigned_url_put_file<'a>(_req: &mut Request, _res: &mut Response) {
    let _file_name = _req.param::<String>("**file_name");
    let _seconds = _req.query::<u32>("seconds");
    if _file_name.is_some() {
        match request_signed_url(_file_name.unwrap(), http::Method::PUT, _seconds).await {
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
