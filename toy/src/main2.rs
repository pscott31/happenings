use app::model::*;
use leptos::*;
use rust_decimal_macros::dec;
use std::env;
use dotenv::dotenv;
use futures::stream::{self, StreamExt}; 


pub async fn list_bookings() -> Result<Vec<Booking>, ServerFnError> {
    use log::*;
    use square_api::SquareApiClient;
    use std::collections::HashMap;
    info!("listing bookings");

    let endpoint = env::var("SQUARE_ENDPOINT").expect("Error: SQUARE_API_KEY variable not found");
    let api_key = env::var("SQUARE_API_KEY").expect("Error: SQUARE_API_KEY variable not found");
    let location_id =
        env::var("SQUARE_LOCATION_ID").expect("Error: SQUARE_API_KEY variable not found");
    let endpoint = format!("https://{}", endpoint);

    let mut client = SquareApiClient::new(&endpoint);
    client.client = client
        .client
        .default_header("Authorization".to_string(), format!("Bearer {}", api_key));

    let customers = client.search_customers().limit(100).await?.customers.unwrap_or_default();

    let customer_map: HashMap<String, square_api::model::Customer> = customers
        .into_iter()
        .map(|c| (c.id.clone().unwrap(), c))
        .collect();

    let query = 
        square_api::model::SearchOrdersQuery {
            filter: Some(square_api::model::SearchOrdersFilter {
                state_filter: Some(square_api::model::SearchOrdersStateFilter { states: vec!["OPEN".to_string()] }),
                source_filter: Some(square_api::model::SearchOrdersSourceFilter {
                    source_names: Some(vec!["StukeleyHappenings".to_string()]),
                }),
                ..Default::default()
                }),
                ..Default::default()
            };


    let resp = client
    .search_orders()
    .location_ids(vec![location_id.clone()])
    .query(query.clone())
    .await?;


    let orders_stream = stream::iter(resp.orders.unwrap_or_default());

    let cmp = &customer_map;

    let arse = orders_stream.then(|o| async move{
        println!("{:?}", o);
        match o.customer_id.as_ref() {
            Some(id) => {
                if let Some(customer) = cmp.get(id) {
                    // If the customer is already in the map, use that
                    // Some(customer.clone())
                    info!("customer {:?} already in map", id);
                } else {
                    // Otherwise, fetch the customer asynchronously
                    info!("customer {:?} not in map trying to fetch", id);
                    let cust = client
                    .retrieve_customer(id)
                    .await;
                    
                    match cust {
                        Ok(resp) => {
                            info!("customer {:?} fetched", id);
                            // Some(resp.customer)
                        },
                        Err(err) => {
                            info!("customer {:?} failed to get info", id);
                        }
                    }
                    
                    
                }
            }
            None => {
                info!("empty customer id!");
            },
        }
    }).collect::<Vec<_>>()
    .await;   
    // .map(|res| {
    //     res.orders.unwrap_or_default()});



    // if let Err(err) = orders {
    //     return Err(err.into())
    // }
    // let orders = orders.unwrap();

    // for o in orders {
    //     println!("{:?}", order);
    //     match o.customer_id.as_ref() {
    //         Some(id) => customer_map.get(id).or_else( || {
    //             // None
    //             info!("don't have customer {:?}, trying to fetch", id);   
    //             let arse = client
    //                 .retrieve_customer(id)
    //                 .await
    //                 .ok()
    //                 .map(|resp| resp.customer);
    //             arse
    //         }),
            
    //         None => None,
    //     };
    // }
    // let orders_stream = futures::stream::iter(orders);


    let dave = client
        .search_orders()
        .location_ids(vec![location_id])
        .query(query)
        .await
        .map(|res| {
            // info!("res: {:?}", res);

            // res.orders.map(|orders| {
            // orders
            res.orders
                .unwrap_or_default()
                .iter()
                // .filter(|&o| o.source.as_ref().is_some_and(|s| s.name.as_ref().is_some_and(|n| n=="StukeleyHappenings")) )
                // .filter(|&o| o.state.as_ref().is_some_and(|s| s !="DRAFT"))
                .map(|o|  {
                    let customer = match o.customer_id.as_ref() {
                        Some(id) => customer_map.get(id).or_else( || {
                            None
                            // info!("don't have customer {:?}, trying to fetch", id);   
                            // client.retrieve_customer(id).await.ok().map(|resp| resp.customer).map(|mc| mc)
                        }),
                        
                        None => None,
                    };
                        
                    // }
                    // if let Some(customer_id) = o.customer_id.as_ref() {
                    //     info!("customer_id: {:?}", customer_id);
                    //     let customer = o.customer_id.as_ref().and_then(|id| customer_map.get(id));
                    // }
                    // if customer.is_none() {
                    //     info!("don't have customer {:?}, trying to fetch", o.customer_id);                        
                    // }

                    // client.retrieve_customer()
                    println!("{:?} {:?} {:?} {:?} {:?}", o.id.clone(), o.state.clone(), o.source.clone(), o.customer_id.clone(), customer);
                    
                    Booking {
                    id: o.id.clone().unwrap(),
                    // id: o.id.cloned().unwrap_or_default(),
                    event_id: "bar".into(),
                    contact: BookingContact {
                        id: "baz".into(),
                        name: "qux".into(),
                        email: "quux".into(),
                        phone_no: "quuz".into(),
                        event_id: "needed?".into(),
                    },
                    tickets: vec![Ticket {
                        booking_id: "foo".into(),
                        ticket_type: TicketType {
                            name: "someticket".into(),
                            price: dec!(1.23),
                            square_item_id: "".into(),
                            square_catalog_version: 0,
                        },
                        vegetarian: false,
                        gluten_free: false,
                        dietary_requirements: "garply".into(),
                    }],
                }})
                .collect::<Vec<_>>()
            // })
        })
        .map_err(|e| {
            warn!("error listing bookings: {}", e.to_string());
            e.into()
        });
    dave

}



#[tokio::main]
async fn main() {
    println!("\n\nOFF WE GO!");
    dotenv().ok();
    let ret = list_bookings().await;
    if let Err(e) = ret {
        println!("Error: {}", e.to_string());
    }    
}
