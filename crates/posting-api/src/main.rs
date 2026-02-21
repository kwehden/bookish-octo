use posting_api::router;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("bind should work");
    axum::serve(listener, router())
        .await
        .expect("server should run");
}
