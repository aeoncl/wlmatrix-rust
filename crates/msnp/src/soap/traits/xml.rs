pub trait TryFromXml : Sized {
    type Error;

    fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error>;
}

pub trait ToXml {
    type Error;
    fn to_xml(&self) -> Result<String, Self::Error>;
}