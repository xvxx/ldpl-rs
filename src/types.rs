//! Type in the LDPL Language.

#[derive(Debug, PartialEq, Clone)]
pub enum LDPLType {
    Number,
    Text,
    List(Box<LDPLType>),
    Map(Box<LDPLType>),
}

impl LDPLType {
    /// Create an LDPLType from an ident like `NUMBER` or `text list`.
    pub fn from(name: &str) -> Self {
        match name.to_lowercase().as_ref() {
            "number" => LDPLType::Number,
            "number list" => LDPLType::List(Box::new(LDPLType::Number)),
            "number map" | "number vector" => LDPLType::Map(Box::new(LDPLType::Number)),
            "text" => LDPLType::Text,
            "text list" => LDPLType::List(Box::new(LDPLType::Text)),
            "text map" | "text vector" => LDPLType::Map(Box::new(LDPLType::Text)),
            _ => unimplemented!(),
        }
    }

    pub fn is_number(&self) -> bool {
        LDPLType::Number == *self
    }

    pub fn is_text(&self) -> bool {
        LDPLType::Text == *self
    }

    pub fn is_list(&self) -> bool {
        if let LDPLType::List(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_map(&self) -> bool {
        if let LDPLType::Map(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_collection(&self) -> bool {
        self.is_list() || self.is_map()
    }

    pub fn is_text_collection(&self) -> bool {
        if let LDPLType::List(inner) = self {
            **inner == LDPLType::Text
        } else if let LDPLType::Map(inner) = self {
            **inner == LDPLType::Text
        } else {
            false
        }
    }

    pub fn is_number_collection(&self) -> bool {
        if let LDPLType::List(inner) = self {
            **inner == LDPLType::Number
        } else if let LDPLType::Map(inner) = self {
            **inner == LDPLType::Number
        } else {
            false
        }
    }
}
