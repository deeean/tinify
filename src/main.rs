use std::collections::HashMap;
use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Error, middleware};
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use tinify::tinify;

#[derive(Debug)]
struct File {
    data: Vec<u8>,
    content_type: Option<String>,
}

async fn ping() -> &'static str {
    "pong"
}

async fn parse_multipart(mut payload: Multipart) -> Result<HashMap<String, File>, Error> {
    let mut res = HashMap::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();

        let content_type = match field.content_type() {
            Some(content_type) => Some(content_type.to_string()),
            None => None,
        };

        let key = match content_disposition.get_name() {
            Some(key) => key.to_string(),
            None => continue,
        };

        let mut buf = Vec::new();

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            buf.extend_from_slice(&data);
        }

        res.insert(key, File {
            data: buf,
            content_type,
        });
    }

    Ok(res)
}

async fn compress(payload: Multipart) -> Result<HttpResponse, Error> {
    let res = match parse_multipart(payload).await {
        Ok(res) => res,
        Err(_) => return Ok(HttpResponse::InternalServerError().into()),
    };

    let image = match res.get("image") {
        Some(image) => image,
        None => return Ok(HttpResponse::InternalServerError().into()),
    };

    let quality = match res.get("quality") {
        Some(quality) => match std::str::from_utf8(&quality.data) {
            Ok(quality) => match quality.parse::<f32>() {
                Ok(quality) => quality,
                Err(_) => return Ok(HttpResponse::InternalServerError().into()),
            },
            Err(_) => return Ok(HttpResponse::InternalServerError().into()),
        },
        None => 70.0,
    };

    let content_type = match &image.content_type {
        Some(content_type) => match content_type.as_str() {
            "image/jpeg" => "image/jpeg",
            "image/png" => "image/png",
            _ => return Ok(HttpResponse::InternalServerError().into()),
        },
        None => return Ok(HttpResponse::InternalServerError().into()),
    };

    match tinify(&image.data, quality) {
        Ok(res) => {
            if res.len() > image.data.len() {
                return Ok(HttpResponse::Ok().content_type(content_type).body(image.data.clone()));
            }

            Ok(HttpResponse::Ok().content_type(content_type).body(res))
        },
        Err(_) => Ok(HttpResponse::InternalServerError().into()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .route("/ping", actix_web::web::get().to(ping))
            .route("/compress", actix_web::web::post().to(compress))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
