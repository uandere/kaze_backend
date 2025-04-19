// Simple end‑to‑end throughput benchmark against a **running** Kaze backend.
//
// Start the server locally (`cargo run -- server …`) and then run
//
// ```bash
// cargo test --test integration_bench -- --nocapture
// ```
//
// Three CSV files will be produced next to the test binary:
//   * `sharing.csv`   – 5 rows, POST /diia/sharing
//   * `generate.csv`  – 5 rows, POST /agreement/generate
//   * `signing.csv`   – 5 rows, POST /diia/signature  +
//                       time until GET /agreement/get_signed succeeds
//
// Every CSV row is:  `idx,duration_ms`                (sharing / generate)
//                    `idx,sign_ms,wait_signed_ms`     (signing)

mod common;
use anyhow::Result;
use chrono::Duration;
use common::Request;
use csv::Writer;
use reqwest::Client;
use tokio::time::{sleep, Instant};

/// where the backend listens
const BASE: &str = "http://127.0.0.1:3000";

/// helper – write a vector of `Duration`s into a CSV (idx, ms)
fn dump_durations(name: &str, durs: &[Duration]) -> Result<()> {
    let mut wtr = Writer::from_path(name)?;
    for (i, d) in durs.iter().enumerate() {
        wtr.write_record(&[i.to_string(), d.num_milliseconds().to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}

/// helper – like above but with two columns
fn dump_signing(name: &str, sign: &[Duration], waits: &[Duration]) -> Result<()> {
    let mut wtr = Writer::from_path(name)?;
    for i in 0..sign.len() {
        wtr.write_record(&[
            i.to_string(),
            sign[i].num_milliseconds().to_string(),
            waits[i].num_milliseconds().to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

#[tokio::test]
async fn bench() -> Result<()> {
    // prepare requests
    let common::Setup {
        sharing_requests,
        generate_requests,
        signing_requests,
    } = common::setup("./tests/mockup_signature", "tests/mockup_users").await?;

    // ─────────────────────────────────────────────────────────────────────
    // 1.  /diia/sharing
    // ─────────────────────────────────────────────────────────────────────
    let mut sharing_durs = Vec::with_capacity(5);
    for req in sharing_requests {
        sharing_durs.push(req.send(BASE).await?);
    }
    dump_durations("sharing.csv", &sharing_durs)?;


    // ─────────────────────────────────────────────────────────────────────
    // 2.  /agreement/generate  (landlord ↔ landlord)
    // ─────────────────────────────────────────────────────────────────────
    let mut generate_durs = Vec::with_capacity(5);
    for req in generate_requests {
        generate_durs.push(req.send(BASE).await?);
    }
    dump_durations("generate.csv", &generate_durs)?;


    // ─────────────────────────────────────────────────────────────────────
    // 3.  /diia/signature  (+ wait for /agreement/get_signed)
    // ─────────────────────────────────────────────────────────────────────
    let mut sign_durs   = Vec::with_capacity(5);
    let mut wait_durs   = Vec::with_capacity(5);
    let client = Client::new();

    for (idx, req) in signing_requests.into_iter().enumerate() {
        // POST /diia/signature
        sign_durs.push(req.send(BASE).await?);

        // then poll /agreement/get_signed until 200
        let url = format!(
            "{BASE}/agreement/get_signed?tenant_id=landlord{0}&landlord_id=landlord{0}\
             &housing_id=housing1&_uid=landlord{0}",
            idx + 1
        );

        let start = Instant::now();
        loop {
            let resp = client.get(&url).send().await?;
            if resp.status().is_success() {
                break;
            }
            sleep(std::time::Duration::from_millis(200)).await;
        }
        wait_durs.push(common::to_chrono(start.elapsed()));
    }
    dump_signing("signing.csv", &sign_durs, &wait_durs)?;

    Ok(())
}
