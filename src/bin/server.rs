use std::env;
use dotenv::dotenv;
use s3_gateway_rs::controller::s3::request_signed_url;
use salvo::Result;
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
    .allow_methods(vec![Method::GET, Method::POST, Method::DELETE, Method::PUT]).into_handler();
    let router = Router::new()
        .hoop(cors_handler)
        .push(
            Router::with_path("api/resources/upload/<file_name>")
                .goal(redirect_upload)
        )
        .push(
            Router::with_path("api/resources/presigned-url/<file_name>")
                .get(get_presigned_url)
        )
        .push(
            Router::with_path("api/resources/<file_name>")
                .get(redirect_download)
        )
        ;
    log::info!("{:#?}", router);
    let acceptor = TcpListener::new(&host).bind().await;
    Server::new(acceptor).serve(router).await;
}

#[handler]
async fn get_presigned_url<'a>(_req: &mut Request, _res: &mut Response) {
    let _file_name = _req.param::<String>("file_name");
    if _file_name.is_some() {
        match request_signed_url(_file_name.unwrap(), http::Method::PUT).await {
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
async fn redirect_upload(_req: &mut Request, _res: &mut Response, _depot: &mut Depot, _ctrl: &mut FlowCtrl) -> Result<()> {
    let _file_name = _req.param::<String>("file_name");
    if _file_name.is_some() {
        match request_signed_url(_file_name.unwrap(), http::Method::PUT).await {
            Ok(url) => {
                Proxy::default_hyper_client(url).handle(_req, _depot, _res, _ctrl).await;
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
    Ok(())
}

#[handler]
async fn redirect_download(_req: &mut Request, _res: &mut Response) -> Result<()> {
    let _file_name: Option<String> = _req.param::<String>("file_name");
    if _file_name.is_some() {
        match request_signed_url(_file_name.unwrap(), http::Method::GET).await {
            Ok(url) => _res.render(Redirect::other(url)),
            Err(error) => {
                _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                _res.render(Json(error.to_string()));
            }
        }
    } else {
        _res.render("File Name is mandatory".to_string());
        _res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok(())
}