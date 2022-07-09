use std::fmt::{
    Display,
    Formatter,
    Result,
};

/// Use Deserialize to convert e.g. from request JSON into Book struct.
use serde::Deserialize;

/// Demo book structure with some example fields for id, title, author.
#[derive(Debug, Deserialize, Clone, Eq, Hash, PartialEq)]
pub struct Book {
    pub id: u32,
    pub title: String,
    pub author: String,
}

/// Display the book using the format "{title} by {author}".
/// This is a typical Rust trait and is not axum-specific.
impl Display for Book {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} by {}", self.title, self.author)
    }
}