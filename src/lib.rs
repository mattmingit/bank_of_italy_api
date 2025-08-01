//! # Data Fetching Module - Banca d'Italia
//!
//! This module provides a convenient wrapper for interacting with the [Banca d'Italia Exchange Rate API](https://www.bancaditalia.it/compiti/operazioni-cambi/portale-tassi/index.html). Here you can find the link to API docs ([Exchange Rate API docs](https://www.bancaditalia.it/compiti/operazioni-cambi/Operating_Instructions.pdf?language_id=1))
//!
//! ## Features
//! - Fetch supported currencies and their associated countries.
//! - Retrieve the latest exchange rates in EUR and USD.
//! - Automatic deserialization into strongly-typed Rust structs.
//!
//! ## Example Usage
//! ```rust
//! use bank_of_italy_api::BancaDItalia;
//!
//! #[tokio::main]
//! async fn main() {
//!     let boi = BancaDItalia::new().unwrap();
//!     let currencies = boi.get_currencies().await.unwrap();
//!     println!("{:#?}", currencies);
//! }
//! ```
use date_utils::{parse_to_datetime, DateTimeError, DateType, OffsetType};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use thiserror::Error;
use time::Date;

/// Represent the Bank of Italy API base url.
const BOI_BASE_URL: &str = "https://tassidicambio.bancaditalia.it/terzevalute-wf-web/rest/v1.0";

/// Generates the URL for fetching the list of currencies.
///
/// This macro expands to a `String` containing the full URL to the `/currencies` endpoint.
macro_rules! currencies_url {
    () => {
        format!("{}/currencies?lang=en", BOI_BASE_URL)
    };
}

/// Generates the URL for fetching the latest exchange rates.
///
/// This macro expands to a `String` containing the full URL to the `/latestRates` endpoint.
macro_rules! latestrate_url {
    () => {
        format!("{}/latestRates?lang=en", BOI_BASE_URL)
    };
}

/// Represents possible errors that can occur when interacting with the Banca d'Italia API.
#[derive(Debug, Error)]
pub enum BancaDItaliaError {
    /// Request to Banca d'Italia servers failed.
    #[error("Request to Banca d'Italia API failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    /// Failed to deserialize the JSON response from the API.
    #[error("Deserializing response from Banca d'Italia API failed: {0}")]
    DeserializeFailed(#[from] serde_json::Error),
    /// The API returned an error in its payload.
    #[error("Banca d'Italia returned api error: {0}")]
    ApiError(String),
    /// No data was returned.
    #[error("Banca d'Italia API returned an empty dataset.")]
    NoResult,
    /// Failed to convert Strpping into Decimal
    #[error("Failed to convert String type into Decimal: {0}")]
    ConversionFailed(#[from] rust_decimal::Error),
}

impl From<DateTimeError> for BancaDItaliaError {
    fn from(err: DateTimeError) -> Self {
        BancaDItaliaError::ApiError(err.to_string())
    }
}

/// A client for interacting with the Banca d'Italia exchange rate and currency information API.
pub struct BancaDItalia {
    /// Represent the client that performs the connection to Banca d'Italia API.
    client: Client,
}

impl BancaDItalia {
    /// Creates a new Banca d'Italia client.
    ///
    /// The function creates a Banca d'Italia client using `Client` from `reqwest` crate.
    ///
    /// ## Returns
    /// - `Ok(Self)`: Returns a BancaDItalia instance, which allows connection to Banca d'Italia servers.
    /// - `Err(BancaDItaliaError)`: If connection to Banca d'Italia fails.
    ///
    /// ## Example
    /// ```rust
    /// use bank_of_italy_api::{BancaDItalia, BancaDItaliaError};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), BancaDItaliaError> {
    ///    let boi = BancaDItalia::new();
    ///     assert!(boi.is_ok());
    ///     Ok(())
    /// }
    /// ```
    pub fn new() -> Result<Self, BancaDItaliaError> {
        Ok(Self {
            client: Client::builder()
                .build()
                .map_err(BancaDItaliaError::RequestFailed)?,
        })
    }

    /// Retrieves data from Banca d'Italia servers.
    ///
    /// The function is a helper function that standardize the data fetching process from Banca d'Italia servers. It returns a
    /// `Vec<DeserializedOwned>` to implement a generic vector of struct. The specific struct fields will be determined by the
    /// struct passed as result.
    ///
    /// ## Arguments
    /// - `url`: The url to data endpoint.
    /// - `access_key`: The access key that allows to access data stored in JSON structure.
    ///
    /// ## Returns
    /// - `Ok(Vec<DeserializeOwned>)`: A vector composed by data structure that can be deserialized without borrowing any data from the deserializer.
    /// - `Err(BancaDItaliaError)`: If the data fetching fails.
    async fn get_data<T: DeserializeOwned>(
        &self,
        url: &str,
        access_key: &str,
    ) -> Result<Vec<T>, BancaDItaliaError> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await?
            .json::<Value>()
            .await?;
        let data = response
            .get(access_key)
            .and_then(Value::as_array)
            .ok_or(BancaDItaliaError::NoResult)?;
        let result = serde_json::from_value(Value::Array(data.to_owned()))?;
        Ok(result)
    }

    /// Retrieves currency data.
    ///
    /// The function retrieves a registry of the currency. It stores them in a vector of `Currency` object. If the data
    /// fetching fails it returns a `BancaDItaliaError`.
    ///
    /// ## Returns
    /// - `Ok(Vec<Currency>)`: A vector containing the listed currencies.
    /// - `Err(BancaDItaliaError)`: If data fetching fails.
    ///
    /// ## Example
    /// ```rust
    /// use bank_of_italy_api::BancaDItalia;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let boi = BancaDItalia::new().unwrap();
    ///     let currencies = boi.get_currencies().await.unwrap();
    ///     println!("{:#?}", currencies);
    /// }
    /// ```
    pub async fn get_currencies(&self) -> Result<Vec<Currency>, BancaDItaliaError> {
        parse_currency(self.get_data(&currencies_url!(), "currencies").await?)
    }

    /// Retrieves the latest exchange rate data.
    ///
    /// The function retrieves the latest exchange rate data for current listed currencies. It stores them in a vector of `LatestRate` object.
    /// If the data fetching fails it returns a `BancaDItaliaError`.
    ///
    /// ## Returns
    /// - `Ok(Vec<LatestRate>)`: A vector containing the latest exchange rate for current liste currencies.
    /// - `Err(BancaDItaliaError)`: If data fetching fails.
    ///
    /// ## Example
    /// ```rust
    /// use bank_of_italy_api::BancaDItalia;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let boi = BancaDItalia::new().unwrap();
    ///     let latest_rates = boi.get_latest_rate().await.unwrap();
    ///     println!("{:#?}", latest_rates);
    /// }
    /// ```
    pub async fn get_latest_rate(&self) -> Result<Vec<LatestRate>, BancaDItaliaError> {
        parse_latest_rates(self.get_data(&latestrate_url!(), "latestRates").await?)
    }
}

/// Represents the information about data returned by the Banca d'Italia API.
#[derive(Debug, Deserialize, Serialize)]
pub struct ResultInfo {
    /// The number of data point returned by the query.
    #[serde(rename = "totalRecords")]
    pub total_records: i32,
    /// The timezone used in the data.
    #[serde(rename = "timezoneReference")]
    pub timezone_reference: String,
    /// Any additional information about data.
    pub notice: String,
}

/// Represents the metadata about the results returned by the API.
#[derive(Debug, Deserialize, Serialize)]
pub struct MetaData {
    /// The results informations.
    #[serde(rename = "resultsInfo")]
    pub results_info: ResultInfo,
}

/// Represents the vector containing the currencies data.
#[derive(Debug, Deserialize, Serialize)]
pub struct Currencies {
    /// The vector containing the currencies listed in the data.
    pub currencies: Vec<Currency>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrencyAPI {
    pub countries: Vec<CountryAPI>,
    #[serde(rename = "isoCode")]
    pub isocode: String,
    pub name: String,
    pub graph: bool,
}

/// Represents the single currency object.
#[derive(Debug, Deserialize, Serialize)]
pub struct Currency {
    /// The country data of the currency.
    pub countries: Vec<Country>,
    /// The isocode of the currency.
    #[serde(rename = "isoCode")]
    pub isocode: String,
    /// The name of the currency.
    pub name: String,
    pub graph: bool,
}

/// Represents country information of the currency listed.
#[derive(Debug, Deserialize, Serialize)]
pub struct Country {
    /// The isocode of the currency.
    #[serde(rename = "currencyISO")]
    pub currencyiso: String,
    /// The country of the currency.
    pub country: String,
    /// The isocode of the country.
    #[serde(rename = "countryISO")]
    pub countryiso: Option<String>,
    /// The validity start date of the currency.
    #[serde(rename = "validityStartDate")]
    pub validity_start_date: Date,
    /// The validity end date of the currency.
    #[serde(rename = "validityEndDate")]
    pub validity_end_date: Option<Date>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CountryAPI {
    /// The isocode of the currency.
    #[serde(rename = "currencyISO")]
    pub currencyiso: String,
    /// The country of the currency.
    pub country: String,
    /// The isocode of the country.
    #[serde(rename = "countryISO")]
    pub countryiso: Option<String>,
    /// The validity start date of the currency.
    #[serde(rename = "validityStartDate")]
    pub validity_start_date: String,
    /// The validity end date of the currency.
    #[serde(rename = "validityEndDate")]
    pub validity_end_date: Option<String>,
}

/// Converts the currencies method's results to use date instead of string.
///
/// The function converts the `CurrencyAPI` struct into a `Currency` struct so it uses date instead of string.
///
/// ## Arguments
/// - `currencies`: The vector resulting after fetching data from Banca d'Italia API.  
///
/// ## Returns
/// - `Ok(Vec<CurrencyAPI>)`: A vector containing the currencies data.
/// - `Err(BancaDItaliaError)`: If the data fetching fails.
fn parse_currency(currencies: Vec<CurrencyAPI>) -> Result<Vec<Currency>, BancaDItaliaError> {
    let result = currencies
        .into_iter()
        .map(|cur| {
            let countries = cur
                .countries
                .into_iter()
                .map(|c| {
                    Ok(Country {
                        currencyiso: c.currencyiso,
                        country: c.country,
                        countryiso: c.countryiso,
                        validity_start_date: parse_to_datetime(
                            &c.validity_start_date,
                            DateType::End,
                            OffsetType::Utc,
                        )?
                        .date(),
                        validity_end_date: c
                            .validity_end_date
                            .as_deref()
                            .map(|d| parse_to_datetime(d, DateType::End, OffsetType::Utc))
                            .transpose()?
                            .map(|date| date.date()),
                    })
                })
                .collect::<Result<Vec<Country>, BancaDItaliaError>>()?;

            Ok(Currency {
                countries,
                isocode: cur.isocode,
                name: cur.name,
                graph: cur.graph,
            })
        })
        .collect::<Result<Vec<Currency>, BancaDItaliaError>>()?;

    Ok(result)
}

/// Represents latest rates data object
#[derive(Debug, Deserialize, Serialize)]
pub struct LatestRate {
    /// The country related to rates data.
    pub country: String,
    // The currency related to rates data.
    pub currency: String,
    /// The isocode of the currency.
    #[serde(rename = "isoCode")]
    pub isocode: String,
    /// The uic code of the currency.
    #[serde(rename = "uicCode")]
    pub uiccode: String,
    /// The exchange rate between currency and euro.
    #[serde(rename = "eurRate")]
    pub eur_rate: Decimal,
    /// The exchange rate between currency and usd.
    #[serde(rename = "usdRate")]
    pub usd_rate: Decimal,
    /// The usd exchange convention.
    #[serde(rename = "usdExchangeConvention")]
    pub usd_exchange_convention: String,
    /// The usd exchange convention code.
    #[serde(rename = "usdExchangeConventionCode")]
    pub usd_exchange_convention_code: String,
    /// The reference date.
    #[serde(rename = "referenceDate")]
    pub reference_date: Date, //OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LatestRateAPI {
    /// The country related to rates data.
    pub country: String,
    // The currency related to rates data.
    pub currency: String,
    /// The isocode of the currency.
    #[serde(rename = "isoCode")]
    pub isocode: String,
    /// The uic code of the currency.
    #[serde(rename = "uicCode")]
    pub uiccode: String,
    /// The exchange rate between currency and euro.
    #[serde(rename = "eurRate")]
    pub eur_rate: String,
    /// The exchange rate between currency and usd.
    #[serde(rename = "usdRate")]
    pub usd_rate: String,
    /// The usd exchange convention.
    #[serde(rename = "usdExchangeConvention")]
    pub usd_exchange_convention: String,
    /// The usd exchange convention code.
    #[serde(rename = "usdExchangeConventionCode")]
    pub usd_exchange_convention_code: String,
    /// The reference date.
    #[serde(rename = "referenceDate")]
    pub reference_date: String,
}

/// Converts the metest rates method's results to use date instead of string.
///
/// The function converts the `LatestRateAPI` struct into a `LatestRate` struct so it uses date instead of string.
///
/// ## Arguments
/// - `latest_rates`: The vector resulting after fetching data from Banca d'Italia API.  
///
/// ## Returns
/// - `Ok(Vec<LatestRateAPI>)`: A vector containing the latest rates data.
/// - `Err(BancaDItaliaError)`: If the data fetching fails.
fn parse_latest_rates(
    latest_rates: Vec<LatestRateAPI>,
) -> Result<Vec<LatestRate>, BancaDItaliaError> {
    latest_rates
        .into_iter()
        .map(|rate| {
            let reference_date =
                parse_to_datetime(&rate.reference_date, DateType::Start, OffsetType::Utc)?.date();
            Ok(LatestRate {
                country: rate.country,
                currency: rate.currency,
                isocode: rate.isocode,
                uiccode: rate.uiccode,
                eur_rate: clean_decimal(&rate.eur_rate)?,
                usd_rate: clean_decimal(&rate.usd_rate)?,
                usd_exchange_convention: rate.usd_exchange_convention,
                usd_exchange_convention_code: rate.usd_exchange_convention_code,
                reference_date,
            })
        })
        .collect()
}

/// Clean the response `String` value to correctly convert it into a `rust_decimal::Decimal`.
///
/// The function converts a `String` input into a `Decimal` number.
///
/// ## Arguments
/// - `input`: The String type number.
///
/// ## Returns
/// - `Ok(Decimal)`: The converted `Decimal` number.
/// - `Err(BancaDItaliaError)`: If the conversion fails.
fn clean_decimal(input: &str) -> Result<Decimal, BancaDItaliaError> {
    let cleaned = input.trim();
    if cleaned == "N.A." {
        return Ok(Decimal::from(0));
    }
    Ok(Decimal::from_str(cleaned)?)
}
