use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    deployment::static_deployment::deploy("minecraft", "../console").await
}
