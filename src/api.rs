#[derive(Debug, PartialEq, Eq)]
pub enum ApiType {
    Types,
    Properties,
    Methods,
    Events,
}

impl std::convert::TryFrom<&str> for ApiType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "types" => Ok(ApiType::Types),
            "properties" => Ok(ApiType::Properties),
            "methods" => Ok(ApiType::Methods),
            "events" => Ok(ApiType::Events),
            _ => Err("Unsupported API type"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Property {
    pub type_name: String,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Enum {
        name: String,
    },
    Data {
        name: String,
    },
    Struct {
        name: String,
        properties: Vec<Property>,
        optional_properties: Vec<Property>,
    },
}

impl Type {
    pub fn new_enum(name: String) -> Self {
        Type::Enum { name }
    }

    pub fn new_data(name: String) -> Self {
        Type::Data { name }
    }

    pub fn new_struct(
        name: String,
        properties: Vec<Property>,
        optional_properties: Vec<Property>,
    ) -> Self {
        Type::Struct {
            name,
            properties,
            optional_properties,
        }
    }
}
