use std::net::TcpListener;

pub fn spawn_server() -> Address {
    let host = "127.0.0.1";
    let listener = TcpListener::bind(format!("{host}:0")).expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::startup::run(listener).expect("Failed to bind address");
    tokio::spawn(server);

    format!("http://{host}:{port}")
}

pub type Address = String;
