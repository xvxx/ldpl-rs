//! Type in the LDPL Language.

#[derive(PartialEq)]
pub enum LDPLType {
    Number,
    Text,
    List(Box<LDPLType>),
    Map(Box<LDPLType>),
}

impl LDPLType {
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
}
