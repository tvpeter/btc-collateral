use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MempoolSpaceFeeRate {
	#[serde(rename = "fastestFee")]
	pub fastest_fee: usize,
	#[serde(rename = "halfHourFee")]
	pub half_hour_fee: usize,
	#[serde(rename = "hourFee")]
	pub hour_fee: usize,
	#[serde(rename = "economyFee")]
	pub economy_fee: usize,
	#[serde(rename = "minimumFee")]
	pub minimum_fee: usize,
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

#[cfg(test)]
mod tests {
	use super::*;

	#[ignore]
	#[test]
	fn test_get_feerate() {
		let data = get_mempool_feerate();

		let fees = match data {
			Ok(fees) => fees,
			Err(error) => panic!("{:?}", error),
		};

		assert_ne!(fees.fastest_fee, fees.economy_fee);
		assert!(fees.fastest_fee > fees.half_hour_fee);
	}
}
