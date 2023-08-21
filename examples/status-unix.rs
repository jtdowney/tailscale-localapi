use std::env;

async fn run() {
    let socket_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "/var/run/tailscale/tailscaled.sock".to_string());
    let client = tailscale_localapi::LocalApi::new_with_socket_path(socket_path);

    dbg!(client.status().await.unwrap());
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(run());
}
