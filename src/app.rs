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

#[component]
pub fn App() -> impl IntoView {
    let TEST_TICKET_TYPES: [TicketType; 2] = [
        TicketType {
            name: "Adult".into(),
            price: dec!(25.00),
        },
        TicketType {
            name: "Child".into(),
            price: dec!(15.00),
        },
    ];

    let (ticket_types, _set_ticket_types) = create_signal(TicketTypes::new(TEST_TICKET_TYPES));
    provide_context(ticket_types);

    let g1 = Ticket::new(
        "Main Booker Person",
        ticket_types.get_untracked().standard().unwrap(),
    );
    let booker = create_rw_signal::<Ticket>(g1);

    let (guests, set_guests) = create_signal::<ReactiveList<Ticket>>(IndexMap::new());
    set_guests.tracked_push(Ticket::new(
        "Joe Bloggs",
        ticket_types.get_untracked().find("Adult").unwrap(),
    ));
    set_guests.tracked_push(Ticket::new(
        "Jane Bloggs",
        ticket_types.get_untracked().find("Child").unwrap(),
    ));

    let badgers = move || {
        guests.with(|gl| {
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
                          <IconButton on_click=move |()| set_guests.tracked_remove(uid) icon=FaTrashSolid>
                            Remove Guest
                          </IconButton>
                        </div>
                      </div>
                    }
                })
                .collect_view()
        })
    };

    view! {
      <section class="section">
        <div class="container">
          <h1 class="title">Little Stukeley Christmas Dinner</h1>
          <p class="subtitle">Get your tickets for the final village event of the year!</p>

          <div class="panel">
            <div class="panel-heading py-2">
              <span>About You</span>
            </div>
            // <div class="panel-block">

            <EmailField/>
            <TicketForm ticket=booker/>

          // </div>

          </div>

          {badgers}

          <IconButton
            icon=FaPlusSolid
            color=Color::Primary
            on_click=move |_| set_guests.tracked_push(Ticket::new("New Guest", ticket_types().standard().unwrap()))
          >
            "Add Guest"
          </IconButton>
        </div>

        <BookingSummary booker=booker guests=guests/>
      </section>
    }
}

#[component]
pub fn BookingSummary(
    #[prop(into)] booker: Signal<Ticket>,
    #[prop(into)] guests: Signal<IndexMap<Uuid, RwSignal<Ticket>>>,
) -> impl IntoView
where
{
    view! {
      <section>
        <div>
          <h2 class="title">Booking Summary</h2>
          <TicketSummary ticket=booker/>
          <For each=move || guests.get() key=|(k, _)| *k let:item>
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
      <p>{move || ticket().name}</p>
      <p>{move || ticket().ticket_type.name}</p>
      <p>{move || ticket().vegetarian}</p>
    }
}

#[component]
pub fn TicketForm(ticket: RwSignal<Ticket>) -> impl IntoView {
    let tt = Signal::derive(move || ticket().ticket_type);
    let set_tt = move |new| ticket.update(|g| g.ticket_type = new);

    let name = Signal::derive(move || ticket().name);
    let set_name = move |new| ticket.update(|g| g.name = new);

    let veg = Signal::derive(move || ticket().vegetarian);
    let set_veg = move |new| ticket.update(|g| g.vegetarian = new);

    let gf = Signal::derive(move || ticket().gluten_free);
    let set_gf = move |new| ticket.update(|g| g.gluten_free = new);

    let reqs = Signal::derive(move || ticket().dietry_requirements);
    let set_reqs = move |new| ticket.update(|g| g.dietry_requirements = new);

    view! {
      <div class="pt-3"></div>
      <NameField get=name set=set_name/>
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

