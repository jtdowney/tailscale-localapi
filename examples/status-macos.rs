use std::{process::Command, str};

const SEARCH: &[u8] = b".tailscale.ipn.macos/sameuserproof-";

fn port_and_password() -> (u16, String) {
    let output = Command::new("lsof")
        .arg("-n")
        .arg("-a")
        .arg(format!("-u{}", unsafe { libc::getuid() }))
        .arg("-c")
        .arg("IPNExtension")
        .arg("-F")
        .output()
        .unwrap();

    let offset = output
        .stdout
        .windows(SEARCH.len())
        .position(|w| w == SEARCH)
        .unwrap();
    let start = offset + SEARCH.len();
    let end = output.stdout[start..]
        .iter()
        .position(|&byte| byte == b'\n')
        .map(|pos| start + pos)
        .unwrap_or(output.stdout.len());
    let port_and_password = str::from_utf8(&output.stdout[start..end]).unwrap();
    let mut parts = port_and_password.split('-');
    let port = parts.next().unwrap().parse().unwrap();
    let password = parts.next().unwrap().to_string();

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
