use crate::components::controls::*;
use crate::components::*;
use crate::model::*;
use crate::reactive_list::*;
// use crate::utils::OptionalMaybeSignal;

use indexmap::IndexMap;
use leptos::logging::*;
use leptos::*;
use leptos_icons::FaIcon::*;
use rust_decimal_macros::dec;
use uuid::Uuid;

fn test_event() -> Event {
    let ticket_types: [TicketType; 2] = [
        TicketType {
            name: "Adult".into(),
            price: dec!(25.00),
            square_item_id: "726AQUKD776RI3W7NQIK3AVZ".into(),
            square_catalog_version: 1700477397626,
        },
        TicketType {
            name: "Child".into(),
            price: dec!(15.00),
            square_item_id: "A2PZMGOAICINGRLJPAYLVSUY".into(),
            square_catalog_version: 1700477397626,
        },
    ];

    Event {
        id: "xmas2023".into(),
        name: "Little Stukeley Christmas Dinner".into(),
        tagline: "Get your tickets for the final village event of the year!".into(),
        ticket_types: TicketTypes::new(ticket_types),
    }
}

#[component]
pub fn App() -> impl IntoView {
    let event = store_value(test_event());
    let default_ticket_type = event().ticket_types.standard().unwrap();

    let raw_booking = Booking::new("", "", event().id);
    let first_ticket = Ticket::new(raw_booking.id.clone(), default_ticket_type);
    let mut raw_tickets = ReactiveList::<Ticket>::new();
    raw_tickets.insert(Uuid::new_v4(), create_rw_signal(first_ticket));

    let (booking, set_booking) = create_signal::<Booking>(raw_booking);
    let ticket_types = store_value(event().ticket_types);
    provide_context(ticket_types);

    let name = Signal::derive(move || booking().name);
    let set_name = move |new| set_booking.update(|b| b.name = new);

    let email = Signal::derive(move || booking().email);
    let set_email = move |new| set_booking.update(|b| b.email = new);

    let phone_no = Signal::derive(move || booking().phone_no);
    let set_phone_no = move |new| set_booking.update(|b| b.phone_no = new);

    let (tickets, set_tickets) = create_signal::<ReactiveList<Ticket>>(IndexMap::new());

    let (modal_active, set_modal_active) = create_signal::<bool>(false);

    let badgers = move || {
        tickets.with(|gl| {
            log!("recomuting badger");
            gl.iter()
                .enumerate()
                .map(|(i, (&uid, &gv))| {
                    view! {
                      <Field label=move || {
                          view! {
                            "Ticket "
                            {i + 1}

                            <IconButton on_click=move || set_tickets.tracked_remove(uid) icon=FaTrashSolid>
                              Remove
                            </IconButton>
                          }
                      }>
                        <TicketControl ticket=gv/>
                      </Field>
                    }
                })
                .collect_view()
        })
    };

    let add_ticket = move || {
        set_tickets
            .tracked_push(Ticket::new(booking().id.clone(), ticket_types().standard().unwrap()))
    };

    async fn gen_pay_link(
        booking: &Booking,
        tickets: &ReactiveList<Ticket>,
    ) -> Result<String, reqwest::Error> {
        // ) -> String {
        let client = reqwest::Client::new();
        let res =
            client
                .post("https://connect.squareupsandbox.com/v2/online-checkout/payment-links")
                .body(
                    r#"{
            "order": {
              "location_id": "L7CEWJ6XCDC38",
              "line_items": [
                {
                  "quantity": "1",
                  "catalog_version": 1700477397626,
                  "catalog_object_id": "VF54IAUH3FRNQMNE7T43ZXUB"
                }
              ]
            },
            "idempotency_key": "1689b61e-8e03-45b5-b882-11e6b916f3a0",
            "description": "Your Xmas Dinner 2023 Tickers"
          }"#,
                )
                .send()
                .await?
                .text()
                .await?;

        // match res {
        //     Ok(resp) => format! {"ok! {:?}", resp},
        //     Err(e) => format! {"ok! {:?}", e},
        // }
        Ok(res)
    }

    let link_action =
        create_action(move |_: &()| {
            let booking = booking().to_owned();
            let tickets = tickets().to_owned();
            async move {
                // todo!()
                gen_pay_link(&booking, &tickets).await
            }
        });

    // async fn arse(_: &()) -> Result<String, String> { Ok("yay".into()) }

    // let link_action = create_action::<O=() F=()>(|_| async { return Ok(()) });w
    //move |()| {
    //|booking: Booking, tickets: ReactiveList<Ticket>| {
    // `task` is given as `&String` because its value is available in `input`
    // gen_pay_link(booking(), tickets())
    // arse()
    // });

    // the most recent returned result
    let result_of_call =
        move || {
            link_action.value().with(|x| match x {
                None => view! { "no result yet" }.into_view(),
                // Some(stuff) => view! { {format!("{}", stuff)} }.into_view(),
                Some(Err(err)) => view! { {format!("{}", err)} }.into_view(),
                Some(Ok(res)) => view! { {format!("{:?}", res)} }.into_view(),
            })
        };
    // whether the call is pending
    let pending = link_action.pending();

    // let todger = link_action.dispatch(());
    let git_paid = move || link_action.dispatch(());
    // let git_paid = move || log!("yay");
    view! {
      <section class="section">
        <div class="container">
          <h1 class="title">Little Stukeley Christmas Dinner</h1>
          <h1>result {result_of_call}</h1>
          <h1>pending {pending}</h1>
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
              <IconButton icon=FaPlusSolid color=Color::Primary on_click=move || set_modal_active(true)>
                "See summary"
              </IconButton>

              <IconButton icon=FaPlusSolid color=Color::Primary on_click=move || link_action.dispatch(())>
                // move || link_action.dispatch(())
                "Proceed to Payment"
              </IconButton>

              <p class="control"></p>
            </div>
          </div>
        </div>

        <div>modal here? {modal_active}</div>
        <Modal
          active=modal_active
          set_active=set_modal_active
          title="Booking Summary"
          footer=move || {
              view! {
                <button class="button is-success">Save changes</button>
                <button class="button">Cancel</button>
              }
          }
        >

          <BookingSummary booking=booking tickets=tickets/>
        </Modal>
      </section>
    }
}

#[component]
pub fn TelNoControl(
    #[prop(into)] get: MaybeSignal<String>,
    #[prop(into)] set: Callback<String>,
) -> impl IntoView {
    let on_change = move |ev: leptos::ev::Event| set(event_target_value(&ev));
    view! {
      <p class="control is-expanded">
        <input class="input" type="tel" placeholder="Phone number (optional)" prop:value=get on:change=on_change/>
      </p>
    }
}
// TODO
// enum ViewOrString{
//   v(ViewFn)
//   s(String)
// }
#[component]
// pub fn HFormField<F, IV>(children: Children, #[prop(optional)] label: F) -> impl IntoView
pub fn Field(children: Children, #[prop(optional, into)] label: ViewFn) -> impl IntoView
where
    // F: Fn() -> IV,
    // IV: IntoView + 'static,
{
    let children = children()
        .nodes
        .into_iter()
        .map(|child| {
            view! { <div class="field">{child}</div> }
        })
        .collect_view();

    // let label = label.and_then(
    //   move |label|     view!{      <label class="label">{label}</label>{/view}}
    // )

    view! {
      <div class="field is-horizontal">
        <div class="field-label is-normal">

          <label class="label">{label.run()}</label>
        </div>
        <div class="field-body">{children}</div>
      </div>
    }
}
#[component]
pub fn BookingSummary(
    #[prop(into)] booking: Signal<Booking>,
    #[prop(into)] tickets: Signal<IndexMap<Uuid, RwSignal<Ticket>>>,
) -> impl IntoView
where
{
    let total = move || {
        tickets()
            .values()
            .fold(dec!(0), |a, t| a + t().ticket_type.price)
    };
    view! {
      <table class="table is-fullwidth">
        <tr>
          <th>Booked By</th>
          <th>Email</th>
          <th>Tel</th>
        </tr>
        <tr>
          <td>{move || booking().name}</td>
          <td>{move || booking().email}</td>
          <td>{move || booking().phone_no}</td>
        </tr>
      </table>
      <p>Tickets</p>
      <p>Booking Email:</p>

      <table class="table is-fullwidth">
        <tr>
          <th>Type</th>
          <th>Notes</th>
          <th>Price</th>
        </tr>

        <For each=move || tickets.get() key=|(k, _)| *k let:item>
          <TicketSummary ticket=item.1/>
        </For>

        <tfoot>
          <th></th>
          <th>Total</th>
          <th>{move || format!("{:.2}", total())}</th>
        </tfoot>
      </table>
    }
}

#[component]
pub fn TicketSummary(#[prop(into)] ticket: Signal<Ticket>) -> impl IntoView {
    let options = move || {
        vec![
            ticket().vegetarian.then_some("vegetarian".to_string()),
            ticket().gluten_free.then_some("gluten free".to_string()),
            if ticket().dietry_requirements.is_empty() {
                None
            } else {
                Some(ticket().dietry_requirements)
            },
        ]
        .into_iter()
        .filter_map(|o| o)
        .collect::<Vec<_>>()
        .join(", ")
    };

    let ticket_type = move || ticket().ticket_type.name;
    let ticket_price = move || ticket().ticket_type.price;

    // <p>{move || item.0.to_string()}</p> --!>
    view! {
      <tr>
        <td>{ticket_type}</td>
        <td>{options}</td>
        <td>{move || ticket_price().to_string()}</td>
      </tr>
    }
}

