#![cfg(unix)]

use std::{env, net::SocketAddr, time::Duration};

use tailscale_localapi::{BackendState, LocalApi, Status, UnixStreamClient, Whois};

const READINESS_TIMEOUT: Duration = Duration::from_secs(30);
const WHOIS_TIMEOUT: Duration = Duration::from_secs(5);

fn local_api() -> LocalApi<UnixStreamClient> {
    let socket = env::var("TAILSCALE_SOCKET")
        .expect("TAILSCALE_SOCKET must identify the Compose LocalAPI socket");
    LocalApi::new_with_socket_path(socket)
}

async fn wait_for_node_b(api: &LocalApi<UnixStreamClient>, deadline: Duration) -> Status {
    let mut last_error = String::from("node-b never appeared in the peer map");
    let readiness = async {
        loop {
            match api.status().await {
                Ok(status) if status.peer.values().any(|peer| peer.hostname == "node-b") => {
                    return status;
                }
                Ok(_) => {
                    last_error = String::from("status succeeded but node-b was absent");
                }
                Err(error) => {
                    last_error = error.to_string();
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    tokio::time::timeout(deadline, readiness)
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for the Headscale network map: {last_error}"))
}

fn node_b_address(status: &Status) -> SocketAddr {
    let node_b = status
        .peer
        .values()
        .find(|peer| peer.hostname == "node-b")
        .expect("node-b disappeared from the peer map");
    let node_b_ip = *node_b
        .tailscale_ips
        .first()
        .expect("node-b has no Tailscale IP");

    SocketAddr::new(node_b_ip, 443)
}

async fn whois_with_timeout(
    api: &LocalApi<UnixStreamClient>,
    address: SocketAddr,
    deadline: Duration,
) -> Whois {
    tokio::time::timeout(deadline, api.whois(address))
        .await
        .expect("timed out requesting Whois")
        .expect("Whois request failed")
}

#[tokio::test]
#[ignore = "requires the tests/headscale Docker Compose environment"]
async fn status_reports_running_tailnet_and_peer() {
    let api = local_api();
    let status = wait_for_node_b(&api, READINESS_TIMEOUT).await;

    assert!(matches!(status.backend_state, BackendState::Running));
    assert!(!status.tailscale_ips.is_empty());
    assert!(status.peer.values().any(|peer| peer.hostname == "node-b"));
}

#[tokio::test]
#[ignore = "requires the tests/headscale Docker Compose environment"]
async fn whois_identifies_peer_and_user() {
    let api = local_api();
    let status = wait_for_node_b(&api, READINESS_TIMEOUT).await;
    let whois = whois_with_timeout(&api, node_b_address(&status), WHOIS_TIMEOUT).await;

    assert_eq!(whois.node.hostinfo.hostname.as_deref(), Some("node-b"));
    assert_eq!(whois.user_profile.login_name, "test");
}
