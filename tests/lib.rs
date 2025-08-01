use bank_of_italy_api::BancaDItalia;

#[tokio::test]
async fn test_get_currencies() {
    let boi = BancaDItalia::new().unwrap();
    let response = boi.get_currencies().await;
    assert!(response.is_ok());
    let result = response.unwrap();
    assert_eq!(result[0].isocode, "ADP");
    assert_eq!(result[0].countries[0].currencyiso, "ADP");
    assert_eq!(result[0].countries[0].country, "ANDORRA");

    assert_eq!(result[2].isocode, "AFN");
    assert_eq!(result[2].countries[0].currencyiso, "AFN");
    assert_eq!(
        result[2].countries[0].country,
        "AFGHANISTAN (Islamic State of)"
    );
}

#[tokio::test]
async fn test_get_latest_rates() {
    let boi = BancaDItalia::new().unwrap();
    let response = boi.get_latest_rate().await;
    assert!(response.is_ok(), "Error: {:#?}", response);
    let result = response.unwrap();
    assert_eq!(result[0].currency, "Afghani");
    assert_eq!(result[0].isocode, "AFN");

    let euro = result.iter().find(|rate| rate.isocode == "EUR").unwrap();
    assert_eq!(euro.isocode, "EUR");
    assert_eq!(euro.uiccode, "242");
    assert_eq!(euro.currency, "Euro");
    assert_eq!(euro.country, "EUROPEAN MONETARY UNION");
}
