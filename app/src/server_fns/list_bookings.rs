use crate::model::*;
use leptos::*;
#[cfg(feature = "ssr")]
use rust_decimal_macros::dec;
use std::env;

#[server(ListBookings, "/api")]
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

    let customers = client.search_customers().limit(500).await?.customers.unwrap_or_default();

    let customer_map: HashMap<String, square_api::model::Customer> = customers
        .into_iter()
        .map(|c| (c.id.clone().unwrap(), c))
        .collect();

    let dave = client
        .search_orders()
        .location_ids(vec![location_id])
        .await
        .map(|res| {
            // info!("res: {:?}", res);

            // res.orders.map(|orders| {
            // orders
            res.orders
                .unwrap_or_default()
                .iter()
                .filter(|&o| o.source.as_ref().is_some_and(|s| s.name.as_ref().is_some_and(|n| n=="StukeleyHappenings")) )
                .map(|o|  {
                    let customer = o.customer_id.as_ref().and_then(|id| customer_map.get(id));

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
    // Ok(dave)
    // Ok(vec![Booking {
    //     id: "foo".into(),
    //     event_id: "bar".into(),
    //     contact: BookingContact {
    //         id: "baz".into(),
    //         name: "qux".into(),
    //         email: "quux".into(),
    //         phone_no: "quuz".into(),
    //         event_id: "needed?".into(),
    //     },
    //     tickets: vec![Ticket {
    //         booking_id: "foo".into(),
    //         ticket_type: TicketType {
    //             name: "someticket".into(),
    //             price: dec!(1.23),
    //             square_item_id: "".into(),
    //             square_catalog_version: 0,
    //         },
    //         vegetarian: false,
    //         gluten_free: false,
    //         dietary_requirements: "garply".into(),
    //     }],
    // }])
}

