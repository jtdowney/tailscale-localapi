use std::{fs, path::PathBuf, str};

const DIR: &str = "/Library/Tailscale";

fn port_and_password() -> (u16, String) {
    let dir = PathBuf::from(DIR);

    let port_path = dir.join("ipnport");
    let port = fs::read_link(port_path)
        .unwrap()
        .to_string_lossy()
        .parse()
        .unwrap();
    let password_path = dir.join(format!("sameuserproof-{port}"));
    let password = fs::read_to_string(password_path)
        .unwrap()
        .trim_end()
        .to_string();

    (port, password)
}

async fn run() {
    let (port, password) = port_and_password();
    let client = tailscale_localapi::LocalApi::new_with_port_and_password(port, password);

    dbg!(client.status().await.unwrap());
}

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(run());
}
