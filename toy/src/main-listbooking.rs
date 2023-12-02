use anyhow::{anyhow, Result};
use app::model::*;
use convert_case::{Case, Casing};
use dotenv::dotenv;
use futures::stream::{self, StreamExt};
use leptos::*;
use rust_decimal::Decimal;
use square_api::model::{SearchOrdersFilter, SearchOrdersQuery, SearchOrdersSourceFilter, SearchOrdersStateFilter};
use std::{env, str::FromStr, sync::Arc};
use tracing::*;

struct Config {
    endpoint: String,
    api_key: String,
    location_id: String,
}

impl Default for Config {
    fn default() -> Self {
        let endpoint = format!(
            "https://{}",
            env::var("SQUARE_ENDPOINT").expect("SQUARE_ENDPOINT to be in envrionment")
        );
        Config {
            endpoint,
            api_key: env::var("SQUARE_API_KEY").expect("SQUARE_API_KEY to be in environment"),
            location_id: env::var("SQUARE_LOCATION_ID")
                .expect("SQUARE_LOCATION_ID to be in environment"),
        }
    }
}

fn get_client(cfg: &Config) -> Arc<square_api::SquareApiClient> {
    let mut client = square_api::SquareApiClient::new(&cfg.endpoint);
    client.client = client.client.default_header(
        "Authorization".to_string(),
        format!("Bearer {}", cfg.api_key),
    );
    Arc::new(client)
}

async fn contact_from_order(
    client: Arc<square_api::SquareApiClient>,
    order: &square_api::model::Order,
) -> BookingContact {
    let id = match order.id {
        Some(ref id) => id.clone(),
        None => {
            warn!("No order ID on order?");
            return BookingContact::default();
        }
    };

    let maybe_customer = client
        .retrieve_customer(&id)
        .await
        .map_err(|e| anyhow!("customer search failed: {}", e))
        .and_then(|resp| resp.customer.ok_or(anyhow!("no customer in response")))
        .inspect_err(|e| warn!("error fetching customer {} {}", id, e.to_string()));

    let customer = match maybe_customer {
        Ok(c) => c,
        Err(_) => {
            return BookingContact {
                id: id.clone(),
                name: id.to_case(Case::Title),
                ..Default::default()
            };
        }
    };

    BookingContact {
        id: customer.id.unwrap_or_default(),
        name: format!(
            "{} {}",
            customer.given_name.unwrap_or_default(),
            customer.family_name.unwrap_or_default()
        ),
        email: customer.email_address.unwrap_or_default(),
        phone_no: customer.phone_number.unwrap_or_default(),
        event_id: "".to_string(),
    }
}

async fn booking_from_order(
    client: Arc<square_api::SquareApiClient>,
    order: &square_api::model::Order,
) -> Booking {
    let contact = contact_from_order(client, &order).await;

    let tickets = order.line_items.iter().flatten().map(|line_item| Ticket {
        booking_id: order.id.clone().unwrap_or_default(),
        ticket_type: TicketType {
            name: line_item.variation_name.clone().unwrap_or_default(),
            price: Decimal::new(
                line_item
                    .base_price_money
                    .as_ref()
                    .and_then(|bp| bp.amount)
                    .unwrap_or_default(),
                2,
            ),
            square_item_id: line_item.catalog_object_id.clone().unwrap_or_default(),
            square_catalog_version: line_item.catalog_version.unwrap_or_default(),
        },
        vegetarian: line_item.metadata_or_default("vegeterrible"),
        gluten_free: line_item.metadata_or_default("gluten_free"),
        dietary_requirements: line_item.metadata_or_default("dietary_requirements"),
    });

    Booking {
        id: order.id.clone().unwrap_or_default(),
        event_id: "".to_string(),
        contact: contact.clone(),
        tickets: tickets.collect(),
    }
}

trait ExtractableMetadata {
    fn metadata_or_default<T>(&self, key: &str) -> T
    where
        T: Default + FromStr,
        <T as FromStr>::Err: std::fmt::Debug;
}

impl ExtractableMetadata for square_api::model::OrderLineItem {
    fn metadata_or_default<T>(&self, key: &str) -> T
    where
        T: Default + FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        self.metadata
            .as_ref()
            .ok_or("no metadata".to_string())
            .and_then(|md| md.get(key).ok_or("key not present".to_string()))
            .and_then(|o| o.as_str().ok_or("value not string".to_string()))
            .and_then(|o| o.parse::<T>().map_err(|e| format!("parse failed: {:?}", e)))
            .unwrap_or_else(|e| {
                warn!(key = key, error = e, "error parsing metadata");
                T::default()
            })
    }
}

pub async fn list_bookings() -> Result<Vec<Booking>, ServerFnError> {
    info!("listing bookings");
    let cfg = Config::default();
    let client = get_client(&cfg);

    let query = SearchOrdersQuery {
        filter: Some(SearchOrdersFilter {
            state_filter: Some(SearchOrdersStateFilter {
                states: vec!["OPEN".to_string()],
            }),
            source_filter: Some(SearchOrdersSourceFilter {
                source_names: Some(vec!["StukeleyHappenings".to_string()]),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resp = client
        .search_orders()
        .location_ids(vec![cfg.location_id.clone()])
        .query(query.clone())
        .await?;

    let bookings = stream::iter(resp.orders.unwrap_or_default())
        .map(|order| {
            let client = client.clone();
            async move { booking_from_order(client, &order).await }
        })
        .buffered(10)
        .collect::<Vec<_>>()
        .await;

    Ok(bookings)
}

#[tokio::main]
async fn main() {
    println!("\n\nOFF WE GO!");
    dotenv().ok();

    // pretty_env_logger::formatted_builder()
    //     .filter(None, log::LevelFilter::Warn)
    //     .filter(Some("toy"), log::LevelFilter::Debug)
    //     .init();

    // let filter = tracing_subscriber::EnvFilter::builder()
    //     .with_default_directive(Level::WARN.into()) // Default level for all modules
    //     .parse_lossy("toy=debug");

    // tracing_subscriber::fmt()
    //     .with_max_level(Level::TRACE)
    //     .with_env_filter(filter)
    //     .init();

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let ret = list_bookings().await;
    if let Err(e) = ret {
        println!("Error: {}", e.to_string());
    }
}

