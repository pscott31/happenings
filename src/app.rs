use crate::components::*;
use crate::model::*;
use crate::reactive_list::*;

use indexmap::IndexMap;
use leptos::logging::*;
use leptos::*;
use leptos_icons::FaIcon::*;
use leptos_icons::{FaIcon, Icon};
use rust_decimal_macros::dec;
use uuid::Uuid;

fn test_event() -> Event {
    let ticket_types: [TicketType; 2] = [
        TicketType {
            name: "Adult".into(),
            price: dec!(25.00),
        },
        TicketType {
            name: "Child".into(),
            price: dec!(15.00),
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

    let (tickets, set_tickets) = create_signal::<ReactiveList<Ticket>>(IndexMap::new());

    let badgers = move || {
        tickets.with(|gl| {
            log!("recomuting badger");
            gl.iter()
                .enumerate()
                .map(|(i, (&uid, gv))| {
                    view! {
                      <div class="panel">
                        <div class="panel-heading py-2">{format!("Guest {}", i + 1)}</div>
                        // <div class="panel-block is-flex-direction-column">

                        <TicketForm ticket=*gv/>

                        // </div>
                        <div class="panel-block is-justify-content-flex-end">
                          <IconButton on_click=move |()| set_tickets.tracked_remove(uid) icon=FaTrashSolid>
                            Remove Guest
                          </IconButton>
                        </div>
                      </div>
                    }
                })
                .collect_view()
        })
    };

    let add_ticket = move |_| {
        set_tickets.tracked_push(Ticket::new(
            booking().id.clone(),
            ticket_types().standard().unwrap(),
        ))
    };

    view! {
      <section class="section">
        <div class="container">
          <h1 class="title">Little Stukeley Christmas Dinner</h1>
          <p class="subtitle">Get your tickets for the final village event of the year!</p>

          <div class="box">
            <NameField get=name set=set_name/>
            <EmailField get=email set=set_email/>
          </div>

          {badgers}

          <IconButton icon=FaPlusSolid color=Color::Primary on_click=add_ticket>
            // on_click=move |_| {
            // set_tickets.tracked_push(Ticket::new("booking().id".into(), ticket_types().standard().unwrap()))
            // }
            "Add Ticket"
          </IconButton>
        </div>
        <BookingSummary booking=booking tickets=tickets/>
      </section>
    }
}

#[component]
pub fn BookingSummary(
    #[prop(into)] booking: Signal<Booking>,
    #[prop(into)] tickets: Signal<IndexMap<Uuid, RwSignal<Ticket>>>,
) -> impl IntoView
where
{
    view! {
      <section>
        <div>
          <h2 class="title">Booking Summary</h2>
          <p>Booking Name: {move || booking().name}</p>
          <p>Booking Email: {move || booking().email}</p>
          <For each=move || tickets.get() key=|(k, _)| *k let:item>
            <p>{move || item.0.to_string()}</p>
            <TicketSummary ticket=item.1/>
          </For>
        </div>
      </section>
    }
}

#[component]
pub fn TicketSummary(#[prop(into)] ticket: Signal<Ticket>) -> impl IntoView {
    view! {
      <p>{move || ticket().ticket_type.name}</p>
      <p>{move || ticket().vegetarian}</p>
    }
}

#[component]
pub fn TicketForm(ticket: RwSignal<Ticket>) -> impl IntoView {
    let tt = Signal::derive(move || ticket().ticket_type);
    let set_tt = move |new| ticket.update(|g| g.ticket_type = new);

    let veg = Signal::derive(move || ticket().vegetarian);
    let set_veg = move |new| ticket.update(|g| g.vegetarian = new);

    let gf = Signal::derive(move || ticket().gluten_free);
    let set_gf = move |new| ticket.update(|g| g.gluten_free = new);

    let reqs = Signal::derive(move || ticket().dietry_requirements);
    let set_reqs = move |new| ticket.update(|g| g.dietry_requirements = new);

    view! {
      <div class="pt-3"></div>
      <TicketTypeField get=tt set=set_tt/>

      <div class="field is-horizontal">
        <div class="field-label">
          <label class="label">Dietary Requirements</label>
        </div>

        <div class="field-body is-flex-direction-column">
          <CheckboxField label="Vegetarian" get=veg set=set_veg/>
          <CheckboxField label="Gluten Free" get=gf set=set_gf/>
          <TextField placeholder="Other (please specify)" get=reqs set=set_reqs/>
        </div>
      </div>
    }
}

#[component]
pub fn CheckboxField(
    #[prop(into)] label: String,
    #[prop(into)] get: Signal<bool>,
    #[prop(into)] set: Callback<bool>,
) -> impl IntoView {
    view! {
      <div class="field is-horizontal">
        <div class="control">
          <label class="checkbox">
            <input type="checkbox" prop:checked=move || get() on:change=move |ev| set(event_target_checked(&ev))/>
            {format!(" {} ", label)}
          </label>
        </div>
      </div>
    }
}

#[component]
pub fn TextField(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into)] get: MaybeSignal<String>,
    #[prop(into)] set: Callback<String>,
    #[prop(into, optional)] icon: Option<FaIcon>,
) -> impl IntoView {
    let icon_view = icon.map(|i| {
        view! {
          <span class=format!("icon is-small is-left")>
            <Icon icon=Icon::from(i)/>
          </span>
        }
    });

    let label_view = label.map(|l| {
        view! { <label class="label">{l}</label> }
    });

    view! {
      <div class="field is-horizontal">
        {label_view} <div class="control" class:has-icons-left=icon_view.is_some()>
          <input
            class="input"
            type="text"
            placeholder=placeholder
            prop:value=get
            on:change=move |ev| set(event_target_value(&ev))
          />
          {icon_view}
        </div>
      </div>
    }
}

