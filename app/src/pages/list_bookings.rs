use log::*;

use crate::server_fns::list_bookings;

use leptos::*;

#[component]
pub fn ListBookings() -> impl IntoView {
    let bookings = create_resource(|| (), |_| async move { list_bookings().await });

    view! {
      <section class="section">
        <div class="container">
          <h1 class="title">Booking List</h1>
          <Suspense fallback=move || {
              view! { <p>"Loading..."</p> }
          }>

            {move || match bookings.get() {
                None => view! { <p>"Loading..."</p> }.into_view(),
                Some(Err(e)) => view! { <p>"Error loading bookings: " {e.to_string()}</p> }.into_view(),
                Some(Ok(bs)) => {
                    view! {
                      <table class="table">
                        <thead>
                          <tr>
                            <th>Booking ID</th>
                            <th>Event ID</th>
                            <th>Contact Name</th>
                            <th>Contact Email</th>
                            <th>Tickets</th>
                          </tr>
                        </thead>
                        <tbody>
                          <For
                            each=move || bs.clone()
                            key=|b| b.id.clone()
                            children=move |b| {
                                view! {
                                  <tr>
                                    <td>{b.id}</td>
                                    <td>{b.event_id}</td>
                                    <td>{b.contact.name}</td>
                                    <td>{b.contact.email}</td>
                                  // <td>{b.tickets}</td>
                                  </tr>
                                }
                            }
                          />

                        </tbody>
                      </table>
                    }
                        .into_view()
                }
            }}

          </Suspense>
        </div>
      </section>
    }
}

