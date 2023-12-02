use rust_decimal::Decimal;
use tracing::*;
use uuid::Uuid;

use crate::model::*;
use crate::server_fns::list_bookings;
use leptos::*;

#[derive(Clone, PartialEq)]
enum Tabs {
    Bookings,
    Tickets,
}

#[component]
pub fn ListBookings() -> impl IntoView {
    let bookings = create_resource(|| (), |_| async move { list_bookings().await });
    let (active_tab, set_active_tab) = create_signal(Tabs::Bookings);

    view! {
      <section class="section">
        <div class="container">
          <Suspense fallback=move || {
              view! { <p>"Loading..."</p> }
          }>

            {move || match bookings.get() {
                None => view! { <p>"Loading..."</p> }.into_view(),
                Some(Err(e)) => view! { <p>"Error loading bookings: " {e.to_string()}</p> }.into_view(),
                Some(Ok(bs)) => {
                    view! {
                      <div class="tabs  is-medium is-boxed">
                        <ul>
                          <li
                            class:is-active=move || { active_tab.get() == Tabs::Bookings }
                            on:click=move |_| set_active_tab(Tabs::Bookings)
                          >
                            <a>Bookings</a>
                          </li>
                          <li
                            class:is-active=move || { active_tab.get() == Tabs::Tickets }
                            on:click=move |_| set_active_tab(Tabs::Tickets)
                          >
                            <a>Tickets</a>
                          </li>
                        </ul>
                      </div>

                      {match active_tab() {
                          Tabs::Bookings => view! { <BookingsTab bookings=bs/> },
                          Tabs::Tickets => view! { <TicketsTab bookings=bs/> },
                      }}
                    }
                        .into_view()
                }
            }}

          </Suspense>
        </div>
      </section>
    }
}

#[component]
fn BookingsTab(bookings: Vec<Booking>) -> impl IntoView {
    let b2 = bookings.clone();
    let total_tickets = move || b2.clone().iter().map(|b| b.tickets.len()).sum::<usize>();
    view! {
      <table class="table">
        <thead>
          <tr>
            <th>Contact Name</th>
            <th>Contact Email</th>
            <th>Tickets</th>
            <th>Price</th>
            <th>Payment Type</th>
            <th>Paid Amount</th>
          </tr>
        </thead>
        <tbody>
          <For
            each=move || bookings.clone()
            key=|b| b.id.clone()
            children=move |b| {
                let price = b.tickets.iter().fold(Decimal::new(0, 2), |a, t| { a + t.ticket_type.price });
                let paid = match b.payment {
                    BookingPayment::NotPaid => Decimal::new(0, 2),
                    BookingPayment::Card(amt) | BookingPayment::Cash(amt) => amt,
                };
                view! {
                  <tr>
                    <td>{b.contact.name}</td>
                    <td>{b.contact.email}</td>
                    <td>{b.tickets.len()}</td>
                    <td>{format!("£{}", price)}</td>
                    <td class:has-text-danger=b.payment
                        == BookingPayment::NotPaid>
                      {match b.payment {
                          BookingPayment::NotPaid => "None".to_string(),
                          BookingPayment::Card(_) => "Card".to_string(),
                          BookingPayment::Cash(_) => "Cash".to_string(),
                      }}

                    </td>

                    <td class:has-text-success=paid == price>{format!("£{}", paid)}</td>

                  </tr>
                }
            }
          />

        </tbody>
        <tfoot>
          <tr>
            <th></th>
            <th>
              <b>Total</b>
            </th>
            <th>{total_tickets()}</th>
          </tr>
        </tfoot>
      </table>
    }
}

#[component]
fn TicketsTab(bookings: Vec<Booking>) -> impl IntoView {
    struct TicketWithBooking {
        booking: Booking,
        ticket: Ticket,
    }

    let bookings = store_value(bookings);

    let tickets = move || {
        bookings()
            .iter()
            .flat_map(|b| b.tickets.iter().map(|t| (b.clone(), t.clone())))
            .collect::<Vec<_>>()
    };

    let total_tickets = move || tickets().len();
    let total_veggie = move || tickets().iter().filter(|t| t.1.vegetarian).count();
    let total_gf = move || tickets().iter().filter(|t| t.1.gluten_free).count();

    view! {
      <table class="table">
        <thead>
          <tr>
            <th>Contact Name</th>
            <th>Ticket Type</th>
            <th>Vegetarian</th>
            <th>Gluten Free</th>
            <th>Other Requirements</th>
          </tr>
        </thead>
        <tbody>
          <For
            each=move || tickets()
            key=|(_, _)| Uuid::new_v4()
            children=move |(b, t)| {
                view! {
                  <tr>
                    <td>{b.contact.name}</td>
                    <td>{t.ticket_type.name}</td>
                    <td>{if t.vegetarian { "yes" } else { "" }}</td>
                    <td>{if t.gluten_free { "yes" } else { "" }}</td>
                    <td>{if t.dietary_requirements != "none" { t.dietary_requirements } else { "".to_string() }}</td>
                  </tr>
                }
            }
          />

        </tbody>
        <tfoot>
          <tr>
            <th>
              <b>Totals</b>
            </th>
            <th>{total_tickets()}</th>
            <th>{total_veggie()}</th>
            <th>{total_gf()}</th>
          </tr>
        </tfoot>
      </table>
    }
}

