use std::net::TcpListener;
use std::time::Duration;

use actix_web::dev::Server;
use actix_web::middleware::{Compress, Logger, NormalizePath};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web, http::header};
use serde::{Deserialize, Serialize};

use crate::state::State;
use crate::database;

fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/urls").route(web::post().to(create_url)))
        .service(web::resource("/urls/delete").route(web::delete().to(delete_url)))
        .service(web::resource("/urls/{id}").route(web::get().to(redirect_url)));
}

async fn not_found_handler(_request: HttpRequest) -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({ "error": "Not found" }))
}

pub fn listen(listener: TcpListener, state: State) -> std::io::Result<Server> {
    let state = web::Data::new(state);
    let create_app = move || {
        let app = App::new().app_data(state.clone());
        app
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(Logger::new(r#"%a "%r" %s %b (%{Content-Length}i %{Content-Type}i) "%{Referer}i" "%{User-Agent}i" %T"#))
            .wrap(Compress::default())
            .wrap(NormalizePath::trim())
            .service(web::scope("/api").configure(api_config))
            .default_service(web::route().to(not_found_handler))
    };
    let server = HttpServer::new(create_app)
        .keep_alive(Duration::from_secs(60))
        .listen(listener)?
        .run();
    Ok(server)
}

/* I'm writing the structs & handlers here to save time & for your reading convenience. */
#[derive(Deserialize, Serialize)]
struct Link {
    id: String,
    url: String
}
#[derive(Deserialize)]
struct LinkId {
    id: String
}

// Create short aliases for URLs
async fn create_url(state: web::Data<State>, body: web::Json<Link>) -> HttpResponse {
    let client = match state.database_client().await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Error connecting to database: {:?}", err);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Error connecting to database" }))
        }
    };

    match database::create_link(&client, &body.id, &body.url).await {
        Ok(_) => {
            let response = Link {
                id: format!("{}", body.id),
                url: format!("{}", body.url)
            };

            HttpResponse::Ok().json(response)
        },
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Error shortening URL" }))
    }
}

// Delete short aliases for URLs
async fn delete_url(state: web::Data<State>, body: web::Json<LinkId>) -> HttpResponse {
    let client = match state.database_client().await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Error connecting to database: {:?}", err);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Error connecting to database" }))
        }
    };

    match database::delete_link(&client, &body.id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "success", "message": "Link deleted" })),
        Err(err) => {
            eprintln!("Error deleting Link: {:?}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Error deleting Link" }))
        }
    }
}

// Redirect all requests for an alias to the full URL
async fn redirect_url(state: web::Data<State>, params: web::Path<LinkId>) -> HttpResponse {
    let id = &params.id;

    let client = match state.database_client().await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("Error connecting to database: {:?}", err);
            return HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Error connecting to database" }))
        }
    };

    match database::get_link(&client, &id).await {
        Ok(url) if !url.is_empty() => {
            HttpResponse::Found().append_header((header::LOCATION, url)).finish()
        },
        Ok(_) => {
            HttpResponse::Ok().into()
        },
        Err(err) => {
            eprintln!("Error redirecting to full URL: {:?}", err);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Error redirecting to full URL" }))
        }
    }
}