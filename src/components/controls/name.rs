use leptos::*;
use leptos_icons::{FaIcon::*, Icon};

#[component]
pub fn Name(
    #[prop(into)] get: MaybeSignal<String>,
    #[prop(into)] set: Callback<String>,
) -> impl IntoView {
    view! {
      <div class="control has-icons-left">
        <input
          class="input"
          type="text"
          placeholder="Name"
          prop:value=get
          on:change=move |ev| set(event_target_value(&ev))
        />
        <span class="icon is-small is-left">
          <Icon icon=Icon::from(FaUserSolid)/>
        </span>
      </div>
    }
}

