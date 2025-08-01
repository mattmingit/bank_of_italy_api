<p align="center">
  <h1 align="center">bank_of_italy_api</h1>
  <p align="center">Rust library for retrieving exchange rate and currency information from the Banca d'Italia Exchange Rate API. Provides strongly-typed access to currency data and conversion rates in EUR and USD.</p>

  <p align="center">
      <a href="https://github.com/mattmingit/bank_of_italy_api/actions">
        <img src="https://github.com/mattmingit/bank_of_italy_api/actions/workflows/release.yml/badge.svg" alt="Build Status">
      </a>
      <img src="https://img.shields.io/badge/version-0.1.0-blue.svg" alt="Version">
      <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License">
   </p>
</p>

---

## ✨ Features

- ✅ Fetch supported currencies and their associated countries
- 💶 Retrieve the latest exchange rates in EUR and USD
- 🧱 Strongly-typed models for safe deserialization
- 📅 Parses date strings into `time::Date` via [`date_utils`](https://github.com/mattmingit/date_utils)
- ❌ Graceful error handling via `thiserror`

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bank_of_italy_api = { git = "https://github.com/your-username/bank_of_italy_api" }
```

## 📥 Binary Releases

Precompiled binaries are available on the [ releases page ](https://github.com/mattmingit/bank_of_italy_api/releases) for major platforms.

## 📚 API Overview

| Function              | Description                                       |
| --------------------- | ------------------------------------------------- |
| `BancaDItalia::new()` | Initializes the HTTP client                       |
| `get_currencies()`    | Retrieves a list of currencies and their metadata |
| `get_latest_rate()`   | Fetches the latest exchange rates in EUR and USD  |

## ❗ Error Handling

All fallible operations return the BancaDItaliaError enum, with variants for:

- RequestFailed — network or HTTP failure

- DeserializeFailed — invalid or unexpected JSON

- ApiError — logical/semantic errors in the API response

- NoResult — no results found

- ConversionFailed — parsing strings into decimals or dates failed

## 🔧 Usage Example

```rust
use bank_of_italy_api::BancaDItalia;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BancaDItalia::new()?;
    let currencies = client.get_currencies().await?;
    let rates = client.get_latest_rate().await?;

    println!("Currencies: {:#?}", currencies);
    println!("Exchange Rates: {:#?}", rates);
    Ok(())
}
```
