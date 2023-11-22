use phonenumber::{country, parse, ParseError, PhoneNumber};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TicketType {
    pub name: String,
    pub price: Decimal,
    pub square_item_id: String,
    pub square_catalog_version: i64,
}

#[derive(Clone, Debug)]
pub struct TicketTypes(Vec<TicketType>);

impl TicketTypes {
    pub fn new<T: AsRef<[TicketType]>>(input: T) -> Self { TicketTypes(input.as_ref().into()) }

    pub fn find<T: AsRef<str>>(self, tt_name: T) -> Option<TicketType>
    where
        std::string::String: PartialEq<T>,
    {
        self.0.into_iter().find(|tt| tt.name == tt_name)
    }

    pub fn standard(&self) -> Option<TicketType> { self.0.get(0).map(|x| x.clone()) }
}

impl IntoIterator for TicketTypes {
    type Item = TicketType;
    type IntoIter = <Vec<TicketType> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub tagline: String,
    pub ticket_types: TicketTypes,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate)]
pub struct BookingContact {
    pub id: String,
    #[validate(length(min = 3))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1), custom = "validate_phone_no")]
    pub phone_no: String,
    pub event_id: String,
}

impl BookingContact {
    pub fn phone_number(&self) -> Result<PhoneNumber, ParseError> {
        parse(Some(country::Id::GB), self.phone_no.clone())
    }
}

fn validate_phone_no(phone_str: &str) -> Result<(), ValidationError> {
    match parse(Some(country::Id::GB), phone_str) {
        Ok(pn) => {
            if pn.is_valid() {
                Ok(())
            } else {
                Err(ValidationError::new("Invalid phone number"))
            }
        }
        Err(_) => Err(ValidationError::new("Invalid phone number")),
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate)]
pub struct NewBooking {
    pub event_id: String,
    #[validate]
    pub contact: BookingContact,
    #[serde(default)]
    pub tickets: Vec<Ticket>,
}

impl BookingContact {
    pub fn new<T, U, V>(name: T, email: U, event_id: V) -> Self
    where
        T: Into<String>,
        U: Into<String>,
        V: Into<String>,
    {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            email: email.into(),
            event_id: event_id.into(),
            phone_no: "".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ticket {
    pub booking_id: String,
    pub ticket_type: TicketType,
    pub vegetarian: bool,
    pub gluten_free: bool,
    pub dietary_requirements: String,
}

impl Ticket {
    pub fn new(booking_id: String, tt: TicketType) -> Self {
        Self {
            booking_id: booking_id,
            ticket_type: tt,
            vegetarian: false,
            gluten_free: false,
            dietary_requirements: "".into(),
        }
    }
}

