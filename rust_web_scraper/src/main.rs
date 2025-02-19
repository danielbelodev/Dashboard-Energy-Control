use reqwest;
use serde::Deserialize;
use tokio_postgres::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.awattar.at/v1/marketdata";

    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    
    let parsed: MarketDataResponse = serde_json::from_str(&body)?;
    println!("{:#?}", parsed);

    let(client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=postgres dbname=energy", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Verbindungsfehler: {}", e);
        }
    });

    for entry in parsed.data {

        let start_time = chrono::NaiveDateTime::from_timestamp_opt(entry.start_timestamp, 0).unwrap();
        let end_time = chrono::NaiveDateTime::from_timestamp_opt(entry.end_timestamp, 0).unwrap();
        
        let market_price_eur_per_kwh = entry.marketprice / 1000.0;  // MWh → kWh
        let final_price = market_price_eur_per_kwh * 1.03 + 0.015;  // 3% + 1.5 Cent/kWh
        let unit = "Euro/KWh";

        client.execute(
            "INSERT INTO market_data (start_time, end_time, market_price_eur_per_kwh, final_price_eur_per_kwh, unit) 
             VALUES ($1, $2, $3, $4, $5)",
            &[&start_time, &end_time, &market_price_eur_per_kwh, &final_price, &unit],
        ).await?;
    }

    println!("Daten erfolgreich gespeichert!");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct MarketDataResponse {
    data: Vec<MarketData>,
}

#[derive(Debug, Deserialize)]
struct MarketData {
    start_timestamp: i64,
    end_timestamp: i64,
    marketprice: f64,
    unit: String,
}
