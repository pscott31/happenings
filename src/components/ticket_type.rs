use crate::model::*;
use leptos::logging::*;
use leptos::*;

#[component]
pub fn TicketTypeField(
    #[prop(into)] get: Signal<TicketType>,
    #[prop(into)] set: Callback<TicketType>,
) -> impl IntoView {
    let ticket_types = use_context::<ReadSignal<TicketTypes>>().expect("there to be ticket types");

    let options = ticket_types
        .get_untracked()
        .clone()
        .into_iter()
        .map(|tt| {
            let is_selected = {
                let tt = tt.clone();
                move || tt.name == get().name
            };
            let option_text = format!("{} - £{}", tt.name, tt.price);
            view! {
              <option selected=is_selected value=tt.name>
                {option_text}
              </option>
            }
        })
        .collect_view();

    view! {
      <div class="field is-horizontal">
        <div class="field-label">
          <label class="label">Ticket Type</label>
        </div>
        <div class="field-body">
          <div class="select">
            <select on:change=move |ev| {
                log!("{}", event_target_value(& ev));
                ticket_types().find(event_target_value(&ev)).and_then(|tt| Some(set(tt)));
            }>{options}</select>
          </div>
        </div>
      </div>
    }
}

