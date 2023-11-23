use crate::model::*;
use leptos::*;
use std::env;

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
use crate::square_api;
use log::*;
use sanitizer::prelude::*;
use std::collections::HashMap;
    }
}

#[server(PaymentLink, "/api")]
pub async fn create_payment_link(booking: NewBooking) -> Result<String, ServerFnError> {
    info!("creating payment link for booking: {:?}", booking);

    let resp = {
        let phone_number = booking
            .contact
            .phone_number()?
            .format()
            .mode(phonenumber::Mode::E164)
            .to_string();

        let req = square_api::CreatePaymentLinkRequest {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            description: "Little Stukeley Christmas Dinner".to_string(),
            order: build_order(&booking),
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

        let req = build_request("online-checkout/payment-links").json(&req);

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

#[server(CreateOrder, "/api")]
pub async fn create_order(booking: NewBooking) -> Result<String, ServerFnError> {
    info!("creating order for booking: {:?}", booking);

    let resp = {
        let req = square_api::CreateOrderRequest {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            order: build_order(&booking),
        };

        info!("req_payload {:?}", req);
        let req = build_request("orders").json(&req);

        info!("request: {:?}", req);

        let res = req.send().await.map_err(|e| {
            warn!("failed to call square api: {}", e);
            e
        })?;

        if res.status().is_success() {
            let parsed_res = res.json::<square_api::CreateOrderResponse>().await?;
            info!("order created: {:?}", parsed_res.order);
            return Ok(parsed_res.order.id);
        }

        let error_body = res.text().await?;
        Err(ServerFnError::ServerError(error_body))
    };
    if let Err(e) = resp.as_ref() {
        warn!("error generating payment link: {}", e.to_string())
    };
    resp
}

#[cfg(feature = "ssr")]
fn build_request(method: &str) -> reqwest::RequestBuilder {
    let endpoint = env::var("SQUARE_ENDPOINT").expect("Error: SQUARE_API_KEY variable not found");
    let api_key = env::var("SQUARE_API_KEY").expect("Error: SQUARE_API_KEY variable not found");

    reqwest::Client::new()
        .post(format!("https://{}/v2/{}", endpoint, method))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", api_key),
        )
}

#[cfg(feature = "ssr")]
fn build_order(booking: &NewBooking) -> square_api::NewOrder {
    let location_id =
        env::var("SQUARE_LOCATION_ID").expect("Error: SQUARE_LOCATION_ID variable not found");

    let item_id = env::var("SQUARE_ITEM_ID").expect("Error: SQUARE_LOCATION_ID variable not found");

    let catalog_version = env::var("SQUARE_CATALOG_VERSION")
        .expect("Error: SQUARE_CATALOG_VERSION variable not found");
    let catalog_version = catalog_version.parse::<i64>().unwrap();

    let mut sanitizer = StringSanitizer::from(booking.contact.name.clone());
    sanitizer.trim().to_snake_case();
    let customer_id = sanitizer.get();

    let line_items = booking
        .tickets
        .iter()
        .map(|t| square_api::NewLineItem {
            quantity: "1".to_string(),
            catalog_version, //todo: t.ticket_type.square_catalog_version,
            catalog_object_id: item_id.clone(), //todo: t.ticket_type.square_item_id.clone(),
            metadata: HashMap::from([
                ("gluten_free".to_string(), t.gluten_free.to_string()),
                ("vegeterrible".to_string(), t.vegetarian.to_string()),
                (
                    "dietary_requirements".to_string(),
                    if t.dietary_requirements.is_empty() {
                        "none".to_string()
                    } else {
                        t.dietary_requirements.clone()
                    },
                ),
            ]),
        })
        .collect::<Vec<_>>();

    square_api::NewOrder {
        customer_id: Some(customer_id),
        location_id,
        line_items,
    }
}

