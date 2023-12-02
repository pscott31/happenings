use crate::model::*;
use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};
use futures::stream::{self, StreamExt};
use leptos::*;
use rust_decimal::Decimal;
#[cfg(feature = "ssr")]
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

#[cfg(feature = "ssr")]
fn get_client(cfg: &Config) -> Arc<square_api::SquareApiClient> {
    let mut client = square_api::SquareApiClient::new(&cfg.endpoint);
    client.client = client.client.default_header(
        "Authorization".to_string(),
        format!("Bearer {}", cfg.api_key),
    );
    Arc::new(client)
}

#[cfg(feature = "ssr")]
async fn contact_from_order(
    client: Arc<square_api::SquareApiClient>,
    order: &square_api::model::Order,
) -> BookingContact {
    let id = match order.customer_id {
        Some(ref id) => id.clone(),
        None => {
            warn!("No customer ID on order?");
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

#[cfg(feature = "ssr")]
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

    let payment = match order.tenders.as_ref() {
        Some(tenders) => BookingPayment::Card(
            tenders
                .iter()
                .map(|t| &t.amount_money)
                .flatten()
                .map(|m| m.amount)
                .flatten()
                .map(|a| Decimal::new(a, 2))
                .sum(),
        ),
        None => BookingPayment::NotPaid,
    };

    Booking {
        id: order.id.clone().unwrap_or_default(),
        event_id: "".to_string(),
        contact: contact.clone(),
        tickets: tickets.collect(),
        payment: payment,
    }
}

trait ExtractableMetadata {
    fn metadata_or_default<T>(&self, key: &str) -> T
    where
        T: Default + FromStr,
        <T as FromStr>::Err: std::fmt::Debug;
}

#[cfg(feature = "ssr")]
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

#[server(ListBookings, "/api")]
pub async fn list_bookings() -> Result<Vec<Booking>, ServerFnError> {
    info!("listing bookings");
    warn!("working?");
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

// #[tokio::main]
// async fn main() {
//     println!("\n\nOFF WE GO!");
//     dotenv().ok();

//     // pretty_env_logger::formatted_builder()
//     //     .filter(None, log::LevelFilter::Warn)
//     //     .filter(Some("toy"), log::LevelFilter::Debug)
//     //     .init();

//     let filter = tracing_subscriber::EnvFilter::builder()
//         .with_default_directive(Level::WARN.into()) // Default level for all modules
//         .parse_lossy("toy=debug");

//     tracing_subscriber::fmt()
//         .with_max_level(Level::TRACE)
//         .with_env_filter(filter)
//         .init();

//     let ret = list_bookings().await;
//     if let Err(e) = ret {
//         println!("Error: {}", e.to_string());
//     }
// }

// // use crate::model::*;
// // use leptos::*;
// // #[cfg(feature = "ssr")]
// // use rust_decimal_macros::dec;
// // use std::env;

// // #[server(ListBookings, "/api")]
// pub async fn list_bookings() -> Result<Vec<Booking>, ServerFnError> {
//     use log::*;
//     use square_api::SquareApiClient;
//     use std::collections::HashMap;
//     info!("listing bookings");

//     let endpoint = env::var("SQUARE_ENDPOINT").expect("Error: SQUARE_API_KEY variable not found");
//     let api_key = env::var("SQUARE_API_KEY").expect("Error: SQUARE_API_KEY variable not found");
//     let location_id =
//         env::var("SQUARE_LOCATION_ID").expect("Error: SQUARE_API_KEY variable not found");
//     let endpoint = format!("https://{}", endpoint);

//     let mut client = SquareApiClient::new(&endpoint);
//     client.client = client
//         .client
//         .default_header("Authorization".to_string(), format!("Bearer {}", api_key));

//     let customers = client.search_customers().limit(500).await?.customers.unwrap_or_default();

//     let customer_map: HashMap<String, square_api::model::Customer> = customers
//         .into_iter()
//         .map(|c| (c.id.clone().unwrap(), c))
//         .collect();

//     let dave = client
//         .search_orders()
//         .location_ids(vec![location_id])
//         .await
//         .map(|res| {
//             // info!("res: {:?}", res);

//             // res.orders.map(|orders| {
//             // orders
//             res.orders
//                 .unwrap_or_default()
//                 .iter()
//                 .filter(|&o| o.source.as_ref().is_some_and(|s| s.name.as_ref().is_some_and(|n| n=="StukeleyHappenings")) )
//                 .map(|o|  {
//                     let customer = o.customer_id.as_ref().and_then(|id| customer_map.get(id));

//                     println!("{:?} {:?} {:?} {:?} {:?}", o.id.clone(), o.state.clone(), o.source.clone(), o.customer_id.clone(), customer);

//                     Booking {
//                     id: o.id.clone().unwrap(),
//                     // id: o.id.cloned().unwrap_or_default(),
//                     event_id: "bar".into(),
//                     contact: BookingContact {
//                         id: "baz".into(),
//                         name: "qux".into(),
//                         email: "quux".into(),
//                         phone_no: "quuz".into(),
//                         event_id: "needed?".into(),
//                     },
//                     tickets: vec![Ticket {
//                         booking_id: "foo".into(),
//                         ticket_type: TicketType {
//                             name: "someticket".into(),
//                             price: dec!(1.23),
//                             square_item_id: "".into(),
//                             square_catalog_version: 0,
//                         },
//                         vegetarian: false,
//                         gluten_free: false,
//                         dietary_requirements: "garply".into(),
//                     }],
//                 }})
//                 .collect::<Vec<_>>()
//             // })
//         })
//         .map_err(|e| {
//             warn!("error listing bookings: {}", e.to_string());
//             e.into()
//         });

//     dave
//     // Ok(dave)
//     // Ok(vec![Booking {
//     //     id: "foo".into(),
//     //     event_id: "bar".into(),
//     //     contact: BookingContact {
//     //         id: "baz".into(),
//     //         name: "qux".into(),
//     //         email: "quux".into(),
//     //         phone_no: "quuz".into(),
//     //         event_id: "needed?".into(),
//     //     },
//     //     tickets: vec![Ticket {
//     //         booking_id: "foo".into(),
//     //         ticket_type: TicketType {
//     //             name: "someticket".into(),
//     //             price: dec!(1.23),
//     //             square_item_id: "".into(),
//     //             square_catalog_version: 0,
//     //         },
//     //         vegetarian: false,
//     //         gluten_free: false,
//     //         dietary_requirements: "garply".into(),
//     //     }],
//     // }])
// }

