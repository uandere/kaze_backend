mod common;
use anyhow::Result;

#[tokio::test]
async fn bench() -> Result<()> {
    // using common code.
    let _setup = common::setup("./tests/mockup_signature", "tests/mockup_users").await?;

    Ok(())
}
