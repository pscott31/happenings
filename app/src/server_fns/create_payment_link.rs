use leptos::*;

use crate::model::*;

#[server(PaymentLink, "/api")]
pub async fn create_payment_link(booking: NewBooking) -> Result<String, ServerFnError> {
    use crate::square_api;
    use log::*;
    use sanitizer::prelude::*;
    use std::collections::HashMap;
    use std::env;

    info!("adding booking: {:?}", booking);

    let endpoint = env::var("SQUARE_ENDPOINT").expect("Error: SQUARE_API_KEY variable not found");
    let api_key = env::var("SQUARE_API_KEY").expect("Error: SQUARE_API_KEY variable not found");

    let location_id =
        env::var("SQUARE_LOCATION_ID").expect("Error: SQUARE_LOCATION_ID variable not found");

    let item_id = env::var("SQUARE_ITEM_ID").expect("Error: SQUARE_LOCATION_ID variable not found");

    let catalog_version = env::var("SQUARE_CATALOG_VERSION")
        .expect("Error: SQUARE_CATALOG_VERSION variable not found");
    let catalog_version = catalog_version.parse::<i64>()?;
    let resp = {
        let client = reqwest::Client::new();

        let mut sanitizer = StringSanitizer::from(booking.contact.name.clone());
        sanitizer.trim().to_snake_case();
        let customer_id = sanitizer.get();

        let phone_number = booking
            .contact
            .phone_number()?
            .format()
            .mode(phonenumber::Mode::E164)
            .to_string();

        let line_items = booking
            .tickets
            .iter()
            .map(|t| square_api::NewLineItem {
                quantity: "1".to_string(),
                catalog_version, //todo: t.ticket_type.square_catalog_version,
                catalog_object_id: item_id.clone(), //todo: t.ticket_type.square_item_id.clone(),
                dietary_requirements: HashMap::from([
                    ("gluten_free".to_string(), t.gluten_free.to_string()),
                    ("vegeterrible".to_string(), t.vegetarian.to_string()),
                    (
                        "dietary_requirements".to_string(),
                        t.dietary_requirements.clone(),
                    ),
                ]),
            })
            .collect::<Vec<_>>();

        let req = square_api::CreatePaymentLinkRequest {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            description: "Little Stukeley Christmas Dinner".to_string(),
            order: square_api::NewOrder {
                customer_id: Some(customer_id),
                location_id,
                line_items,
            },
            checkout_options: Some(square_api::CheckoutOptions {
                allow_tipping: false,
                ask_for_shipping_address: false,
                enable_coupon: false,
                enable_loyalty: false,
            }),
            pre_populated_data: Some(square_api::PrePopulatedData {
                buyer_address: None,
                buyer_email: Some(booking.contact.email),
                buyer_phone_number: Some(phone_number),
            }),
        };
        let req = client
            .post(format!(
                "https://{}/v2/online-checkout/payment-links",
                endpoint
            ))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", api_key),
            )
            .json(&req);

        info!("request: {:?}", req);

        let res = req.send().await.map_err(|e| {
            warn!("failed to call square api: {}", e);
            e
        })?;

        if res.status().is_success() {
            let parsed_res = res.json::<square_api::Welcome>().await?;
            return Ok(parsed_res.payment_link.long_url);
        }

        let error_body = res.text().await?;
        Err(ServerFnError::ServerError(error_body))
    };
    if let Err(e) = resp.as_ref() {
        warn!("error generating payment link: {}", e.to_string())
    };
    resp
}

