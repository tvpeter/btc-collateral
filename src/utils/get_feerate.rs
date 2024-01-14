use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MempoolSpaceFeeRate {
	#[serde(rename = "fastestFee")]
	fastest_fee: u16,
	#[serde(rename = "halfHourFee")]
	half_hour_fee: u16,
	#[serde(rename = "hourFee")]
	hour_fee: u16,
	#[serde(rename = "economyFee")]
	economy_fee: u16,
	#[serde(rename = "minimumFee")]
	minimum_fee: u16,
}

#[tokio::main]
pub async fn get_mempool_feerate() -> Result<MempoolSpaceFeeRate, String> {
	let response = reqwest::get("https://mempool.space/api/v1/fees/recommended").await;
	let result = match response {
		Ok(res) => res,
		Err(error) => return Err(format!("Error fetching feerates: {:?}", error)),
	};

	let data: Result<MempoolSpaceFeeRate, reqwest::Error> = result.json().await;
	match data {
		Ok(rates) => Ok(rates),
		Err(err) => Err(format!("Error serializing fees: {:?}", err)),
	}
}
