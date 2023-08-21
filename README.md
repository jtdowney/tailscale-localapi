# tailscale-localapi

This is a rust crate designed to interact with the [Tailscale](https://tailscale.com) local API. On Linux and other Unix-like systems, this is through a unix socket. On macOS and Windows, this is through a local TCP port and a password. The Tailscale localapi is large but so far this crate does:

1. Get the status of the node and the tailnet (similar to `tailscale status`)
2. Get a certificate and key for the node (similar to `tailscale cert`)
3. Get whois information for a given IP address in the tailnet

## Limitations

This crate uses hyper and requires tokio and async rust.

## Example

```rust
let socket_path = "/var/run/tailscale/tailscaled.sock";
let client = tailscale_localapi::LocalApi::new_with_socket_path(socket_path);
dbg!(client.status().await.unwrap());
```
