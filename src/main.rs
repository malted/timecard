use std::env::var;
use serde_json::{json, Value};
use timecard::chronicle_elastic_client;
use elasticsearch::http::request::JsonBody;

static HOST: &str = "https://10.101.8.1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv::dotenv().ok();

    let client = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
		.cookie_store(true)
        .build()
        .expect("a built client");

	let body = json!({
		"username": var("UNIFI_USER")?,
		"password": var("UNIFI_PASS")?,
	});

    client.post(format!("{HOST}/api/auth/login"))
        .json(&body)
        .send()
        .await?;

    let devices = client.get(format!("{HOST}/proxy/network/v2/api/site/default/clients/active?includeTrafficUsage=true&includeUnifiDevices=true"))
        .send()
        .await?
        .json::<Value>()
        .await?;

	println!("{} devices are online.", devices.as_array().expect("the devices array").len());

	let chronicle = chronicle_elastic_client()?;

	let prepared_devices: Vec<JsonBody<Value>> = devices
		.as_array()
		.expect("the devices array")
		.iter()
		.flat_map(|obj| vec![json!({ "index": { } }).into(), json!(obj).into()])
		.collect();

	let ch = chronicle
		.bulk(elasticsearch::BulkParts::Index("search-hcb-unifi"))
		.body(prepared_devices)
		.send()
		.await?
		.text()
		.await?;

	println!("{}", ch);

    Ok(())
}

