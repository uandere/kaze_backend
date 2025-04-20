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
use std::thread::sleep;

use anyhow::Result;
use chrono::Duration;
use common::Request;
use csv::Writer;
use futures::future::join_all;
use http::header::AUTHORIZATION;
use reqwest::Client;
use tokio::time::Instant;
use tracing::{error, info};

use serde::Deserialize;

#[derive(Deserialize)]
struct StatusResponse {
    status: String,
}

/// where the backend listens
const BASE: &str = "https://www.kazeapi.uk";

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

#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt::try_init();

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

    sleep(std::time::Duration::from_secs(5));

    // ─────────────────────────────────────────────────────────────────────
    // 3a. Fire off all /diia/signature requests in parallel
    // ─────────────────────────────────────────────────────────────────────
    let mut sign_durs = Vec::with_capacity(5);
    let client = Client::new();

    // collect futures
    let sign_futs = signing_requests
        .into_iter()
        .map(|req| req.send(BASE))
        .collect::<Vec<_>>();

    // race them all
    let results = join_all(sign_futs).await;
    for res in results {
        sign_durs.push(res?);
    }

    // ─────────────────────────────────────────────────────────────────────
    // 3b. Now poll /agreement/get_signed for each in turn
    // ─────────────────────────────────────────────────────────────────────
    let mut wait_durs = Vec::with_capacity(sign_durs.len());
for idx in 0..sign_durs.len() {
    // build the URL exactly as your status endpoint expects
    let url = format!(
        "{BASE}/agreement/status?tenant_id=landlord{0}&landlord_id=landlord{0}\
         &housing_id=housing1&_uid=landlord{0}",
        idx + 1
    );

    let start = Instant::now();
    loop {
        let resp = client
            .get(&url)
            .header(AUTHORIZATION, "Bearer dummy_token")
            .send()
            .await?;

        let status = resp.status();

        if status.is_success() {
            let body: StatusResponse = resp.json().await?;
            if body.status == "Signed" {
                info!("GET {} -> Signed", url);
                break;
            }
        } else {
            let text = resp.text().await.unwrap_or_default();
            error!("GET {} -> {} error, body:\n{}", url, status, text);
        }
    }

    wait_durs.push(common::to_chrono(start.elapsed()));
}


    dump_signing("signing.csv", &sign_durs, &wait_durs)?;

    Ok(())
}
