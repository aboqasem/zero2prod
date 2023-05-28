use crate::domain::{EmailAddress, PersonalName};

#[derive(serde::Deserialize)]
pub struct RawSubscriber {
    pub name: String,
    pub email: String,
}

#[derive(serde::Deserialize)]
pub struct Subscriber {
    pub name: PersonalName,
    pub email: EmailAddress,
}

impl TryFrom<RawSubscriber> for Subscriber {
    type Error = String;

    fn try_from(raw_subscriber: RawSubscriber) -> Result<Self, Self::Error> {
        let name = PersonalName::parse(raw_subscriber.name)?;
        let email = EmailAddress::parse(raw_subscriber.email)?;

        Ok(Self { name, email })
    }
}
