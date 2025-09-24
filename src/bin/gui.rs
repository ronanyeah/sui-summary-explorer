use clap::Parser;

#[derive(Parser)]
struct Args {
    folder: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    sui_summary_explorer::gui::main(args.folder).await?;

    Ok(())
}
