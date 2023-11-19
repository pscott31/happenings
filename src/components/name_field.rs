use leptos::*;
use leptos_icons::{FaIcon::*, Icon};

#[component]
pub fn NameField(
    #[prop(into)] get: MaybeSignal<String>,
    #[prop(into)] set: Callback<String>,
) -> impl IntoView {
    view! {
      <div class="field is-horizontal">
        <div class="field-label">
          <label class="label">Name</label>
        </div>
        <div class="field-body">
          <div class="field">
            <div class="control has-icons-left">
              <input
                class="input"
                type="text"
                placeholder="Joe Bloggs"
                prop:value=get
                on:change=move |ev| set(event_target_value(&ev))
              />
              <span class="icon is-small is-left">
                <Icon icon=Icon::from(FaEnvelopeSolid)/>
              </span>
            </div>
          </div>
        </div>
      </div>
    }
}

