use anyhow::Result;


#[tokio::main]
async fn main() -> Result<()> {
    deployment::static_deployment::deploy("minecraft/fov-calculator", "../fov-calculator").await
}
