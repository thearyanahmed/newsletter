use newsletter::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to port.");
    //.expect("failed to bind.");

    let port = listener.local_addr().unwrap().port();

    println!("running on {}:{}","http://127.0.0.1".to_string(),port);

    run(listener)?.await
}
