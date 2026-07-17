# tailscale-localapi

This is a rust crate designed to interact with the [Tailscale](https://tailscale.com) local API. The Tailscale localapi is large but so far this crate does:

1. Get the status of the node and the tailnet (similar to `tailscale status`)
2. Get a certificate and key for the node (similar to `tailscale cert`)
3. Get whois information for a given IP address in the tailnet

## Connections

The connection depends on the Tailscale installation:

| Platform                           | Transport                    | Constructor                            |
| ---------------------------------- | ---------------------------- | -------------------------------------- |
| Linux and other Unix installations | Unix socket                  | `LocalApi::new_with_socket_path`       |
| Windows                            | Named pipe                   | `LocalApi::new_with_named_pipe_path`   |
| Sandboxed macOS                    | Loopback TCP with a password | `LocalApi::new_with_port_and_password` |

## Limitations

This crate uses hyper and requires tokio and async rust.

## Unix example

```rust
let socket_path = "/var/run/tailscale/tailscaled.sock";
let client = tailscale_localapi::LocalApi::new_with_socket_path(socket_path);
dbg!(client.status().await.unwrap());
```
