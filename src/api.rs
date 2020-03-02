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
    type_name: String,
    is_array: bool,
    name: String,
}

impl Property {
    pub fn new(type_name: String, name: String) -> Self {
        let type_info: Vec<_> = type_name.split(" ").collect();

        if name.contains(" ") {
            dbg!(&type_name);
            panic!("invalid name");
        }

        if type_info.len() == 1 {
            Property {
                type_name: type_name,
                is_array: false,
                name,
            }
        } else if type_info.len() == 3 && type_info[0] == "array" && type_info[1] == "of" {
            Property {
                type_name: type_info[2].to_owned(),
                is_array: true,
                name,
            }
        } else if (type_info.len() == 3 && type_info[0] == "enum" && type_info[1] == "of")
            || (&type_info[1..]).iter().any(|&w| w == "or")
        {
            Property {
                type_name: "object".to_owned(),
                is_array: false,
                name,
            }
        } else {
            dbg!(&type_name);
            panic!("unsupported type");
        }
    }

    pub fn rustify_type(&self) -> &str {
        match self.type_name.as_str() {
            "integer" => "isize",
            "boolean" => "bool",
            "string" => "&str",
            _ => &self.type_name,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Type {
    name: String,
    kind: TypeKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum TypeKind {
    Enum,
    Data,
    Struct {
        properties: Vec<Property>,
        optional_properties: Vec<Property>,
        methods: Vec<Method>,
        events: Vec<Event>,
    },
}

impl Type {
    pub fn new_enum(name: String) -> Self {
        Type {
            name,
            kind: TypeKind::Enum,
        }
    }

    pub fn new_data(name: String) -> Self {
        Type {
            name,
            kind: TypeKind::Data,
        }
    }

    pub fn new_struct(
        name: String,
        properties: Vec<Property>,
        optional_properties: Vec<Property>,
        methods: Vec<Method>,
        events: Vec<Event>,
    ) -> Self {
        Type {
            name,
            kind: TypeKind::Struct {
                properties,
                optional_properties,
                methods,
                events,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Argument {
    kind: ArgumentKind,
    optioned: bool,
}

impl Argument {
    pub fn new_property(property: Property, optioned: bool) -> Argument {
        Argument {
            kind: ArgumentKind::Property { property },
            optioned,
        }
    }

    pub fn new_callback(callback: Method, optioned: bool) -> Argument {
        Argument {
            kind: ArgumentKind::Callback { callback },
            optioned,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArgumentKind {
    Property { property: Property },
    Callback { callback: Method },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Method {
    name: String,
    args: Vec<Argument>,
}

impl Method {
    pub fn new(name: String, args: Vec<Argument>) -> Method {
        Method { name, args }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Event {
    event_name: String,
    add_listener: Method,
}

impl Event {
    pub fn new(event_name: String, add_listener: Method) -> Event {
        Event {
            event_name,
            add_listener,
        }
    }
}
