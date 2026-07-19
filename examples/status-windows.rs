#[cfg(windows)]
mod platform {
    use std::env;

    const DEFAULT_PIPE_PATH: &str = r"\\.\pipe\ProtectedPrefix\Administrators\Tailscale\tailscaled";

    async fn run() {
        let pipe_path = env::args()
            .nth(1)
            .unwrap_or_else(|| DEFAULT_PIPE_PATH.to_string());
        let client = tailscale_localapi::LocalApi::new_with_named_pipe_path(pipe_path);

        dbg!(client.status().await.unwrap());
    }

    pub fn main() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(run());
    }
}

fn main() {
    #[cfg(windows)]
    platform::main();
}
