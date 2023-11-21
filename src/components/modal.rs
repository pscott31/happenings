use leptos::*;

#[component]
pub fn Modal(
    #[prop(into)] active: MaybeSignal<bool>,
    #[prop(into)] set_active: Callback<bool>,
    #[prop(into)] title: MaybeSignal<String>,
    children: Children,
    #[prop(into)] footer: ViewFn,
) -> impl IntoView {
    view! {
      <div class="modal" class:is-active=active>
        <div class="modal-background"></div>
        <div class="modal-card">
          <header class="modal-card-head">
            <p class="modal-card-title">{title}</p>
            <button class="delete" aria-label="close" on:click=move |_| set_active(false)></button>
          </header>
          <section class="modal-card-body">{children()}</section>
          <footer class="modal-card-foot">{footer.run()}</footer>
        </div>
      </div>
    }
}

