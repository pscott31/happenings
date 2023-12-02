use std::env;

use anyhow::Result;
use app::model::*;
use css_inline::CSSInliner;
use dotenv::dotenv;
use leptos::*;
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use pluralizer::pluralize;
use rust_decimal::Decimal;
use tracing::*;

struct EmailConfig {
    host: String,
    port: u16,
    user: String,
    password: String,
}

impl Default for EmailConfig {
    fn default() -> Self {
        EmailConfig {
            host: env::var("EMAIL_HOST").expect("EMAIL_HOST to be in environment"),
            user: env::var("EMAIL_USER").expect("EMAIL_USER to be in environment"),
            password: env::var("EMAIL_PASSWORD").expect("EMAIL_PASSWORD to be in environment"),
            port: env::var("EMAIL_PORT")
                .expect("EMAIL_PORT to be in environment")
                .parse()
                .expect("email port number to be u16"),
        }
    }
}

pub async fn email_booking(booking: Booking) -> Result<(), ServerFnError> {
    let cfg = EmailConfig::default();
    let tickets = booking.tickets.clone();

    let tickets_table = view! {
      <table class="table">
        <thead>
          <tr>
            <th>Ticket Type</th>
            <th>Vegetarian</th>
            <th>Gluten Free</th>
            <th>Notes</th>
          </tr>
        </thead>
        <tbody>
          {tickets
              .iter()
              .map(|t| {
                  view! {
                    <tr>
                      <td>{t.ticket_type.name.clone()}</td>
                      <td>{if t.vegetarian { "yes" } else { "" }}</td>
                      <td>{if t.gluten_free { "yes" } else { "" }}</td>
                      <td>
                        {if t.dietary_requirements != "none" { t.dietary_requirements.clone() } else { "".to_string() }}
                      </td>
                    </tr>
                  }
              })
              .collect_view()}
        </tbody>
      </table>
    };

    let booking_table = view! {
      <table class="table">
        <thead>
          <tr>
            <th>Name</th>
            <th>EMail</th>
            <th>Phone</th>
          </tr>
        </thead>
        <tbody>
          <td>{booking.contact.name.clone()}</td>
          <td>{booking.contact.email.clone()}</td>
          <td>{booking.contact.phone_no.clone()}</td>
        </tbody>
      </table>
    };

    let email_view =
        view! {
          <div class="box">
            <div class="block">
              <h2 class="subtitle">Booking By</h2>
              {booking_table}
            </div>
            <div class="block">
              <h2 class="subtitle">Tickets</h2>
              {tickets_table}
            </div>
          </div>
        };

    let rendered = leptos::ssr::render_to_string(|| email_view);
    let css = include_str!("../../style/bulma.css");
    let styled = CSSInliner::options()
        .extra_css(Some(css.into()))
        .build()
        .inline(rendered.as_ref())?;

    let message = MessageBuilder::new()
        .from(("Philip Scott", "safetyfirstphil@gmail.comewhere"))
        .to(vec![("Philip Scott", "phil@safetyphil.com")])
        .subject(format!(
            "Xmas Dinner: {} booked by {}",
            pluralize("ticket", tickets.len().try_into()?, true),
            booking.contact.name.clone(),
        ))
        .html_body(styled)
        .text_body("Switch to HTML View");

    SmtpClientBuilder::new(cfg.host, cfg.port)
        .implicit_tls(false)
        .credentials((cfg.user, cfg.password))
        .connect()
        .await?
        .send(message)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
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
        println!("Error: {}", e);
    }
}

