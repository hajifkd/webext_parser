#[derive(Debug, PartialEq, Eq)]
pub struct Namespace {
    name: String,
    types: Vec<Type>,
    properties: Vec<Property>,
    methods: Vec<Method>,
}

impl Namespace {
    pub(crate) fn new(
        name: String,
        types: Vec<Type>,
        mut properties: Vec<Property>,
        methods: Vec<Method>,
        events: Vec<Event>,
    ) -> Self {
        properties.extend(events.into_iter().map(Event::into));
        Namespace {
            name,
            types,
            properties,
            methods,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn types(&self) -> &[Type] {
        &self.types
    }

    pub fn properties(&self) -> &[Property] {
        &self.properties
    }

    pub fn methods(&self) -> &[Method] {
        &self.methods
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ApiType {
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
pub struct Element {
    type_name: String,
    is_array: bool,
    name: String,
}

impl Element {
    pub fn new(type_name: String, name: String) -> Self {
        let type_info: Vec<_> = type_name.split(" ").collect();

        if name.contains(" ") {
            dbg!(&type_name);
            panic!("invalid name");
        }

        if type_info.len() == 1 {
            Element {
                type_name: type_name,
                is_array: false,
                name,
            }
        } else if type_info.len() == 3 && type_info[0] == "array" && type_info[1] == "of" {
            Element {
                type_name: type_info[2].to_owned(),
                is_array: true,
                name,
            }
        } else if (type_info.len() == 3 && type_info[0] == "enum" && type_info[1] == "of")
            || (&type_info[1..]).iter().any(|&w| w == "or")
        {
            Element {
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
            "number" => "f64",
            "boolean" => "bool",
            "string" => "&str",
            _ => &self.type_name,
        }
    }

    pub fn is_array(&self) -> bool {
        self.is_array
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Type {
    name: String,
    kind: TypeKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypeKind {
    Enum,
    Data,
    Struct {
        elements: Vec<Element>,
        optional_elements: Vec<Element>,
        methods: Vec<Method>,
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

    pub(crate) fn new_struct(
        name: String,
        elements: Vec<Element>,
        optional_elements: Vec<Element>,
        mut methods: Vec<Method>,
        events: Vec<Event>,
    ) -> Self {
        methods.extend(events.into_iter().map(|e| Method {
            name: format!("{}.{}", e.event_name, e.add_listener.name),
            args: e.add_listener.args,
        }));
        Type {
            name,
            kind: TypeKind::Struct {
                elements,
                optional_elements,
                methods,
            },
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Argument {
    kind: ArgumentKind,
    optioned: bool,
}

impl Argument {
    pub fn new_element(element: Element, optioned: bool) -> Argument {
        Argument {
            kind: ArgumentKind::Element { element },
            optioned,
        }
    }

    pub fn new_callback(callback: Method, optioned: bool) -> Argument {
        Argument {
            kind: ArgumentKind::Callback { callback },
            optioned,
        }
    }

    pub fn is_optional(&self) -> bool {
        self.optioned
    }

    pub fn kind(&self) -> &ArgumentKind {
        &self.kind
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArgumentKind {
    Element { element: Element },
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn args(&self) -> &[Argument] {
        &self.args
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Event {
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

impl std::convert::Into<Property> for Event {
    fn into(self) -> Property {
        Property {
            name: self.event_name,
            kind: PropertyKind::Object {
                methods: vec![self.add_listener],
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Property {
    name: String,
    kind: PropertyKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PropertyKind {
    Immediate { type_name: String },
    Object { methods: Vec<Method> },
}

impl Property {
    pub fn new_immediate(name: String, type_name: String) -> Property {
        Property {
            name,
            kind: PropertyKind::Immediate { type_name },
        }
    }

    pub fn new_object(name: String, methods: Vec<Method>) -> Property {
        Property {
            name,
            kind: PropertyKind::Object { methods },
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> &PropertyKind {
        &self.kind
    }
}
