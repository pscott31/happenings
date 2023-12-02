use anyhow::{anyhow, Result};
use app::model::*;
use convert_case::{Case, Casing};
use dotenv::dotenv;
use futures::stream::{self, StreamExt};
use leptos::*;
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use rust_decimal::Decimal;
use square_api::model::{SearchOrdersFilter, SearchOrdersQuery, SearchOrdersSourceFilter, SearchOrdersStateFilter};
use std::{env, str::FromStr, sync::Arc};
use tracing::*;
use uuid::Uuid;

// #[component]
pub fn BookingSummary(booking: Booking) -> impl IntoView
where
{
    let tickets = move || {
        booking
            .tickets
            .iter()
            .cloned()
            .enumerate()
            .collect::<Vec<_>>()
    };

    view! {
      <section>
        <div>
          <h2 class="title">Booking Summary</h2>
          <For each=tickets key=|_| Uuid::new_v4().to_string() let:item>
            <p>yay</p>
          // <p>{move || item.0.to_string()}</p>
          // <TicketSummary ticket=item.1/>
          </For>
        </div>
      </section>
    }
}

// #[component]
pub fn TicketSummary(ticket: Ticket) -> impl IntoView {
    view! {
      <p>{move || ticket.ticket_type.name.clone()}</p>
      <p>{move || ticket.vegetarian}</p>
    }
}

pub async fn email_booking(b: Booking) -> Result<(), ServerFnError> {
    let tv = b
        .tickets
        .iter()
        .map(move |ticket| {
            view! {
              <p>{ticket.ticket_type.name.clone()}</p>
              <p>Vegetarian? {ticket.vegetarian}</p>
              <p>Gluten Free? {ticket.gluten_free}</p>
              <p>Other Dietary Requirements? {ticket.dietary_requirements}</p>
            }
            .into_view()
        })
        .collect::<Vec<_>>();

    let v = view! {
      <h1>New Booking from: {b.contact.name.clone()}</h1>
      <p>Email: {b.contact.email.clone()}</p>
      <p>Phone: {b.contact.phone_no.clone()}</p>
      <h2>Tickets</h2>
      {tv}
    };

    let m = leptos::ssr::render_to_string(|| v);

    info!("{:?}", m);
    let message = MessageBuilder::new()
        .from(("Philip Scott", "safetyfirstphil@gmail.comewhere"))
        .to(vec![("Philip Scott", "phil@safetyphil.com")])
        .subject("Hi!")
        .html_body(m)
        .text_body("Hello world!");

    SmtpClientBuilder::new("smtp.gmail.com", 587)
        .implicit_tls(false)
        .credentials(("safetyfirstphil@gmail.com", "lmrn ivej ahim ncti"))
        .connect()
        .await?
        .send(message)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("\n\nOFF WE GO!");
    dotenv().ok();

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(Level::WARN.into()) // Default level for all modules
        .parse_lossy("toy=debug");

    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(filter)
        .init();

    let test_ticket = Ticket {
        booking_id: "abc".to_string(),
        vegetarian: true,
        gluten_free: false,
        dietary_requirements: "only cheese".to_string(),

        ticket_type: TicketType {
            name: "Adult".to_string(),
            price: Decimal::new(15, 2),
            square_item_id: "foo".to_string(),
            square_catalog_version: 42,
        },
    };

    let b = Booking {
        id: "fancyid".to_string(),
        event_id: "awesomevent".to_string(),
        contact: BookingContact {
            id: "contactid".to_string(),
            name: "The Rock".to_string(),
            email: "the@rock.com".to_string(),
            event_id: "fancyid".to_string(),
            phone_no: "123456".to_string(),
        },
        payment: BookingPayment::NotPaid,
        tickets: vec![test_ticket],
    };

    let ret = email_booking(b).await;
    if let Err(e) = ret {
        println!("Error: {}", e.to_string());
    }
}

