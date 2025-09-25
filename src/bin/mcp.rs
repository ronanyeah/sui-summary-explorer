use clap::Parser;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use sui_summary_explorer::service::SuiService;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = 9393)]
    port: u16,
    #[arg(short, long, default_value = "./package_summaries")]
    summaries_folder: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pkg_path = std::path::Path::new(&args.summaries_folder);
    let packages = sui_summary_explorer::PackageTree::new(&pkg_path)?;

    let bind_address = format!("127.0.0.1:{}", args.port);

    println!("target folder: {}", args.summaries_folder);
    println!("bind address: {}", bind_address);

    let service = StreamableHttpService::new(
        move || Ok(SuiService::new(packages.clone())),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(bind_address).await?;
    axum::serve(tcp_listener, router).await?;

    Ok(())
}
