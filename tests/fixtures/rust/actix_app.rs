use actix_web::{web, App, HttpServer, HttpResponse};

async fn get_users() -> HttpResponse {
    HttpResponse::Ok().body("users")
}

async fn create_user() -> HttpResponse {
    validate_user();
    HttpResponse::Created().body("created")
}

fn validate_user() {
    println!("validating");
}

async fn error_handler(err: String) -> HttpResponse {
    HttpResponse::InternalServerError().body(format!("error: {}", err))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
