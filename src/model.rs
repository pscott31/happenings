use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Default)]
pub struct Booking {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone_no: String,
    pub event_id: String,
}

impl Booking {
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

#[derive(Clone, Debug)]
pub struct Ticket {
    pub booking_id: String,
    pub ticket_type: TicketType,
    pub vegetarian: bool,
    pub gluten_free: bool,
    pub dietry_requirements: String,
}

impl Ticket {
    pub fn new(booking_id: String, tt: TicketType) -> Self {
        Self {
            booking_id: booking_id,
            ticket_type: tt,
            vegetarian: false,
            gluten_free: false,
            dietry_requirements: "".into(),
        }
    }
}

