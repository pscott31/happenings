use rust_decimal::Decimal;

#[derive(Clone, Debug)]
pub struct TicketType {
    pub name: String,
    pub price: Decimal,
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
pub struct Booking {
    pub booker: Ticket,
    pub guests: Vec<Ticket>,
}

#[derive(Clone, Debug)]
pub struct Ticket {
    pub name: String,
    pub ticket_type: TicketType,
    pub vegetarian: bool,
    pub gluten_free: bool,
    pub dietry_requirements: String,
}

impl Ticket {
    pub fn new(name: &str, ticket_type: TicketType) -> Self {
        Self {
            name: name.into(),
            ticket_type,
            vegetarian: false,
            gluten_free: false,
            dietry_requirements: "".into(),
        }
    }
}

