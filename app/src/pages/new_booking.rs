use log::*;

use crate::components::controls::*;
use crate::components::*;
use crate::model::*;
use crate::reactive_list::*;
use crate::server_fns::{create_order, create_payment_link};

use leptos::*;
use leptos_icons::FaIcon::*;
use rust_decimal_macros::dec;
use uuid::Uuid;
use validator::Validate;

fn test_event() -> Event {
    let ticket_types: [TicketType; 1] = [TicketType {
        name: "Adult".into(),
        price: dec!(15.00),
        square_item_id: "VF54IAUH3FRNQMNE7T43ZXUB".into(),
        square_catalog_version: 1700477397626,
    }];

    Event {
        id: "xmas2023".into(),
        name: "Little Stukeley Christmas Dinner".into(),
        tagline: "Get your tickets for the final village event of the year!".into(),
        ticket_types: TicketTypes::new(ticket_types),
    }
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

    let build_booking = move || {
        let contact = booking_contact().to_owned();
        let tickets = tickets()
            .iter()
            .map(|(_, t)| t().to_owned())
            .collect::<Vec<_>>();
        NewBooking {
            event_id: event().id,
            contact,
            tickets,
        }
    };

    let link_action = create_action(move |_: &()| {
        let new_booking = build_booking();
        async move { create_payment_link(new_booking).await }
    });

    let create_order = create_action(move |_: &()| {
        let new_booking = build_booking();
        async move { create_order(new_booking).await }
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
              <PhoneNumber get=phone_no set=set_phone_no/>
            </Field>
            {badgers}

            <div class="field is-grouped">
              <p class="control">
                <IconButton icon=FaPlusSolid color=Color::Secondary on_click=add_ticket>
                  "Add another ticket to your booking"
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

              <IconButton
                disabled=is_invalid
                icon=FaPlusSolid
                color=Color::Primary
                on_click=move || create_order.dispatch(())
              >
                {move || { if pending() { "Creating Order..." } else { "Create Order without Paying" } }}
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
            <pre style="white-space: pre-wrap;">{error_data}</pre>
          </div>
        </Modal>

      </section>
    }
}

