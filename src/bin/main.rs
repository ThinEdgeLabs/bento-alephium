#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");
    println!("Server is ready and running on http://0.0.0.0:8080");

    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .route("/", actix_web::web::get().to(|| async { "Hello!" }))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}