use std::collections::HashMap;
use std::env;

use log::*;

use crate::components::controls::*;
use crate::components::*;
use crate::model::*;
use crate::reactive_list::*;
use crate::square_api;
// use crate::utils::OptionalMaybeSignal;

use indexmap::IndexMap;
use leptos::*;
use leptos_icons::FaIcon::*;
use rust_decimal_macros::dec;
use sanitizer::prelude::*;
use uuid::Uuid;
use validator::Validate;

fn test_event() -> Event {
    let ticket_types: [TicketType; 1] = [
        TicketType {
            name: "Adult".into(),
            price: dec!(15.00),
            square_item_id: "VF54IAUH3FRNQMNE7T43ZXUB".into(),
            square_catalog_version: 1700477397626,
        },
        // TicketType {
        //     name: "Child".into(),
        //     price: dec!(15.00),
        //     square_item_id: "A2PZMGOAICINGRLJPAYLVSUY".into(),
        //     square_catalog_version: 1700477397626,
        // },
    ];

    Event {
        id: "xmas2023".into(),
        name: "Little Stukeley Christmas Dinner".into(),
        tagline: "Get your tickets for the final village event of the year!".into(),
        ticket_types: TicketTypes::new(ticket_types),
    }
}

#[server(PaymentLink, "/api")]
pub async fn add_todo(booking: NewBooking) -> Result<String, ServerFnError> {
    info!("adding booking: {:?}", booking);

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
                catalog_version: catalog_version, //todo: t.ticket_type.square_catalog_version,
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
                location_id: location_id,
                line_items: line_items,
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
        let res = client
            .post("https://connect.squareupsandbox.com/v2/online-checkout/payment-links")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", api_key),
            )
            .json(&req)
            .send()
            .await
            .map_err(|e| {
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

#[component]
pub fn NewBooking() -> impl IntoView {
    let event = store_value(test_event());
    let default_ticket_type = event().ticket_types.standard().unwrap();

    let raw_booking = BookingContact::new("", "", event().id);
    let first_ticket = Ticket::new(raw_booking.id.clone(), default_ticket_type);
    let mut raw_tickets = ReactiveList::<Ticket>::new();
    raw_tickets.insert(Uuid::new_v4(), create_rw_signal(first_ticket));

    let (booking_contact, set_booking_contact) = create_signal::<BookingContact>(raw_booking);
    let ticket_types = store_value(event().ticket_types);
    provide_context(ticket_types);

    let name = Signal::derive(move || booking_contact().name);
    let set_name = move |new| set_booking_contact.update(|b| b.name = new);

    let email = Signal::derive(move || booking_contact().email);
    let set_email = move |new| set_booking_contact.update(|b| b.email = new);

    let phone_no = Signal::derive(move || booking_contact().phone_no);
    let set_phone_no = move |new| set_booking_contact.update(|b| b.phone_no = new);

    // let (tickets, set_tickets) = create_signal::<ReactiveList<Ticket>>(IndexMap::new());
    let (tickets, set_tickets) = create_signal::<ReactiveList<Ticket>>(raw_tickets);

    let (error_seen, set_error_seen) = create_signal::<usize>(0);

    let badgers = move || {
        tickets.with(|gl| {
            debug!("recomuting badger");
            gl.iter()
                .enumerate()
                .map(|(i, (&uid, &gv))| {
                    if i == 0 {
                        view! {
                          <Field label=move || format!("Ticket {}", { i + 1 })>
                            <TicketControl ticket=gv/>
                          </Field>
                        }
                    } else {
                        view! {
                          <Field label=move || {
                              view! {
                                {format!("Ticket {}", { i + 1 })}
                                <br/>
                                <IconButton on_click=move || set_tickets.tracked_remove(uid) icon=FaTrashSolid/>
                              }
                          }>
                            <TicketControl ticket=gv/>
                          </Field>
                        }
                    }
                })
                .collect_view()
        })
    };

    let add_ticket = move || {
        set_tickets.tracked_push(Ticket::new(
            booking_contact().id.clone(),
            ticket_types().standard().unwrap(),
        ))
    };

    let link_action = create_action(move |_: &()| {
        let contact = booking_contact().to_owned();
        let tickets = tickets()
            .iter()
            .map(|(_, t)| t().to_owned())
            .collect::<Vec<_>>();
        let new_booking = NewBooking {
            event_id: event().id,
            contact: contact,
            tickets: tickets,
        };
        async move {
            // todo!()
            // gen_pay_link(&booking, &tickets).await
            add_todo(new_booking).await
        }
    });

    let error_data = move || {
        link_action.value().with(|x| {
            if let Some(Err(err)) = x {
                Some(err.to_string())
            } else {
                None
            }
        })
    };

    let _navigate_to_payment = create_effect(move |_| {
        link_action.value().with(|x| {
            if let Some(Ok(res)) = x {
                let _ = window().location().set_href(res);
            }
        })
    });

    let validation = move || booking_contact().validate();
    let is_invalid = Signal::derive(move || validation().is_err());
    let validation_error = move || {
        if let Err(err) = validation() {
            err.to_string()
        } else {
            "".to_string()
        }
    };
    // whether the call is pending
    let pending = link_action.pending();

    view! {
      <section class="section">
        <div class="container">
          <h1 class="title">Little Stukeley Christmas Dinner</h1>
          <p class="subtitle">Get your tickets for the final village event of the year!</p>

          <div class="box">
            <Field label=|| "Booking Contact">
              <Name get=name set=set_name/>
              <Email get=email set=set_email/>
            </Field>
            <Field>
              <TelNoControl get=phone_no set=set_phone_no/>
            </Field>
            {badgers}

            <div class="field is-grouped">
              <p class="control">
                <IconButton icon=FaPlusSolid color=Color::Secondary on_click=add_ticket>

                  "Add Ticket"
                </IconButton>
              </p>

              <IconButton
                disabled=is_invalid
                icon=FaPlusSolid
                color=Color::Primary
                on_click=move || link_action.dispatch(())
              >
                {move || { if pending() { "Generating Link..." } else { "Proceed to Payment" } }}
              </IconButton>

              <p class="control"></p>
            </div>
          </div>
        </div>

        <Modal
          active=move || error_data().is_some() && link_action.version()() != error_seen()
          close_requested=move || set_error_seen(link_action.version()())
          title="Oh dear"
          footer=move || {
              view! {}
          }
        >

          <div class="block">Something went wrong trying to generate a payment link for you to buy your tickets.</div>
          <div class="block">
            Terribly sorry about that. Could you please let me (Phil Scott) know and tell me what it says below and I will get it sorted
          </div>
          <div class="block">
            <pre style="white-space: pre-wrap;">{move || error_data()}</pre>
          </div>
        </Modal>

      </section>
    }
}

#[component]
pub fn TelNoControl(
    #[prop(into)] get: Signal<String>,
    #[prop(into)] set: Callback<String>,
) -> impl IntoView {
    let on_change = move |ev: leptos::ev::Event| set(event_target_value(&ev));

    let is_valid = move || {
        get.with(
            |s| match phonenumber::parse(Some(phonenumber::country::Id::GB), s) {
                Ok(pn) => pn.is_valid(),
                Err(_) => false,
            },
        )
    };

    let error_msg = move || {
        (!is_valid())
            .then_some(view! { <p class="help is-danger">Please enter a valid phone number</p> })
    };

    // let is_valid = Signal::derive(move || {

    // });

    view! {
      <p class="control is-expanded">
        <input class="input" type="tel" placeholder="Phone number (optional)" prop:value=get on:change=on_change/>
      </p>
      <div>{error_msg}</div>
    }
}

#[component]
pub fn Field(children: Children, #[prop(optional, into)] label: ViewFn) -> impl IntoView
where
{
    let children = children()
        .nodes
        .into_iter()
        .map(|child| {
            view! { <div class="field">{child}</div> }
        })
        .collect_view();

    view! {
      <div class="field is-horizontal">
        <div class="field-label is-normal">

          <label class="label">{label.run()}</label>
        </div>
        <div class="field-body">{children}</div>
      </div>
    }
}

