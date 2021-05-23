extern crate pretty_env_logger;
#[macro_use]
extern crate log;
use std::convert::Infallible;
use std::net::SocketAddr;

mod static_files;
use qrcode::QrCode;
use image::Luma;
extern crate open;



use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};

use hyper::{header, Body, Request, Response, Server, StatusCode};
use ngrok;



async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    log::info!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );
    Ok(req)
}


async fn error_handler(
    err: Box<(dyn std::error::Error + Send + Sync + 'static)>,
    _: RequestInfo,
) -> Response<Body> {
    log::error!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}


pub async fn default_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Access the app state.
    
    return match Response::builder()
        .status(StatusCode::from_u16(200).unwrap())
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from(r#"<!DOCTYPE html>
        <html>
        
        <head>
        
            <style>
                body {
                    background-color: #000
                }
        
                .img-container {
                    width: 60vw;
                    min-width: 360px;
                    text-align: center;
                    margin: 5% auto;
                    background-color: #fff;
                }
            </style>
        </head>
        
        <body>
            <div class="container">
                <div class="img-container">
                    <img 
                        src="./qr_code.png"
                        sizes="(max-width: 512px) 50vw, 60vw">
                </div>
            </div>
        </body>
        
        </html>
        "#))
    {
        Ok(r) => Ok(r),
        _ => Ok(Response::new(Body::from(String::from("OK")))),
    };

}

pub async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Access the app state.
    let default = String::from("index.html");
    let filename = req.param("filename").unwrap_or(&default);

    let (content, mime) = match static_files::get_file_contents_with_mime(filename) {
        Ok(c) => c,
        Err(_e) => {
            return match Response::builder()
                .status(StatusCode::from_u16(404).unwrap())
                .header(header::CONTENT_TYPE, "text/html")
                .body(Body::from(r#"<p>Not Found</p>"#))
            {
                Ok(r) => Ok(r),
                _ => Ok(Response::new(Body::from(String::from("OK")))),
            };
        }
    };

    let mime = &mime.first_or_octet_stream();
    let mime = mime.essence_str();

    log::info!("Loading file: {}", mime);

    match Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, String::from(mime))
        .body(Body::from(content))
    {
        Ok(r) => Ok(r),
        _ => Ok(Response::new(Body::from(String::from("OK")))),
    }
}

fn router() -> Router<Body, Infallible> {

    Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/", default_handler) // Show QR code for ngrok url + /app.rune 
        .get("/:filename", home_handler) 
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let router = router();
    let service = RouterService::new(router).unwrap();

    let server = Server::bind(&addr).serve(service);
    let res = ngrok::builder()
        // server protocol
        .http()
        // the port
        .port(3000)
        .run();

    let t = match res {
        Ok(t) => Some(t),
        _ => None,
    };

    let t = t.unwrap();

    let public_url = match t.https() {
        Ok(url) => Some(url),
        _ => None,
    };
    let public_url  = public_url.unwrap();
    let url_to_open = public_url.as_str();
    let public_url = format!("{}static.rune", public_url.as_str());
    
    
    let code = QrCode::new(public_url.clone()).unwrap();
    let image = code.render::<Luma<u8>>().build();

    image.save("static/qr_code.png").unwrap();

    log::info!("NGROK = {}", url_to_open);
    
    
    match open::that(url_to_open) {
        Ok(exit_status) => {
            log::info!("Opened browser URL with status={}", exit_status)
        },
        Err(_) => {
            log::info!("Couldn't open browser url")
        }
    }
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    // Run this server for... forever!
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}