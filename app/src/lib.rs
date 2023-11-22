use leptos::*;
use leptos_meta::*;
use leptos_router::*;

pub mod components;
pub mod error_template;
pub mod model;
mod pages;
pub mod reactive_list;
pub mod square_api;
pub mod utils;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
      <Stylesheet id="leptos" href="/pkg/happenings.css"/>

      // sets the document title
      <Title text="Welcome to Leptos"/>

      // content for this welcome page
      <Router>
        <main>
          <Routes>
            <Route path="" view=pages::NewBooking/>
          </Routes>
        </main>
      </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
      <h1>"Welcome to Leptos!"</h1>
      <button on:click=on_click>"Click Me: " {count}</button>
    }
}

