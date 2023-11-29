use app::model::*;
use leptos::*;
use rust_decimal_macros::dec;
use std::{env, sync::Arc};
use dotenv::dotenv;
use futures::stream::{self, StreamExt}; 
use log::*;

use square_api::model::{SearchOrdersQuery, SearchOrdersFilter, SearchOrdersStateFilter, SearchOrdersSourceFilter};

struct Config{
    endpoint: String,
    api_key: String,
    location_id: String,
}

impl Default for Config {
    fn default() -> Self {
        let endpoint = format!("https://{}", env::var("SQUARE_ENDPOINT").expect("SQUARE_ENDPOINT to be in envrionment"));
        Config {
            endpoint,
            api_key: env::var("SQUARE_API_KEY").expect("SQUARE_API_KEY to be in environment"),
            location_id: env::var("SQUARE_LOCATION_ID").expect("SQUARE_LOCATION_ID to be in environment"),
        }
    }
}

fn get_client(cfg: &Config) -> Arc<square_api::SquareApiClient> {
    let mut client = square_api::SquareApiClient::new(&cfg.endpoint);
    client.client = client
        .client
        .default_header("Authorization".to_string(), format!("Bearer {}", cfg.api_key));
    Arc::new(client)
}

pub async fn list_bookings() -> Result<Vec<Booking>, ServerFnError> {
    info!("listing bookings");
    let cfg = Config::default();
    let client = get_client(&cfg);

    let query = 
        SearchOrdersQuery {
            filter: Some(SearchOrdersFilter {
                state_filter: Some(SearchOrdersStateFilter { states: vec!["OPEN".to_string()] }),
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


    let orders_stream = stream::iter(resp.orders.unwrap_or_default());

    let customers = orders_stream
        // .filter_map(|o| async move {o.customer_id})
        .then( |o| {
            info!("order id: {:?}", o.id);
            let client_dave = client.clone();

            async move {
            match &o.customer_id {
                Some(id) => {
                    info!("customer id {:?}", id);
                        client_dave.retrieve_customer(&id)
                        .await
                        .map_or_else(|_| Some(id.clone()), 
                        |r| 
                        r
                        .customer
                        .and_then( |c| c.given_name ))
                    
                },
                None =>{
                    info!("no customer id");
                    None
                }
            }   
        }
            // match o.customer_id {
            //     Some(id) => {
            //         async move {
            //             client_dave.retrieve_customer(&id)
            //             .await
            //             .unwrap()
            //             .customer                           
            //         }
            //     },
            //     None => async move {None}
            // }
            
            // async move{
            //     client_dave.retrieve_customer(&id)
            //     .await
            //     .unwrap()
            //     .customer                           
            //}
        });

    let arse = customers
    .collect::<Vec<_>>()
    .await;

    info!("arse: {:?}", arse);
    Ok(vec![])
}



#[tokio::main]
async fn main() {
    println!("\n\nOFF WE GO!");
    dotenv().ok();

    pretty_env_logger::formatted_builder()
    .filter(None, LevelFilter::Warn)
    .filter(Some("toy"), LevelFilter::Debug)
    .init();

    let ret = list_bookings().await;
    if let Err(e) = ret {
        println!("Error: {}", e.to_string());
    }    
}
