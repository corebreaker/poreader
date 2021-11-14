//! Translation catalogues are key part of any localization infrastructure. They contain the lists
//! of messages from the application, possibly disambiguated with identifiers or contexts, and
//! corresponding translations.
//!
//! Catalogs are usually stored in one of two formats: [Portable Objects (`.po`)][PO], used
//! primarily by [GNU gettext][gettext], and [XML Localisation Interchange File Format
//! (XLIFF)][XLIFF], a more generic OASIS open standard.
//!
//! These formats can be converted to each other, and to and from many others, using
//! [translate-toolkit][tt].
//!
//! [XLIFF] is quite flexible and can be used in different ways, but this library focuses
//! primarily on using it in a way [gettext] and [translate-toolkit][tt] work, namely with separate
//! catalogue for each language.
//!
//! [PO]: https://www.gnu.org/software/gettext/manual/html_node/PO-Files.html
//! [XLIFF]: https://www.oasis-open.org/committees/xliff/
//! [gettext]: https://www.gnu.org/software/gettext/
//! [tt]: http://toolkit.translatehouse.org/

extern crate locale_config;
extern crate regex;

mod po;
mod enums;
pub mod unit;
pub mod note;
pub mod error;
pub mod plural;
pub mod comment;

pub use self::{
    po::{PoReader, PoParser},
    enums::{State, Origin, Message},
};

use locale_config::LanguageRange;
use std::collections::HashMap;

/// Catalogue reader.
///
/// Defines common interface of catalogue readers. Read the units by simply iterating over the
/// reader. The other methods are for the important metadata.
pub trait CatalogueReader : Iterator<Item = Result<unit::Unit, error::Error>> {
    fn target_language(&self) -> &LanguageRange<'static>;
    fn header_notes(&self) -> &Vec<note::Note>;
    fn header_comments(&self) -> &Vec<comment::Comment>;
    fn header_properties(&self) -> &HashMap<String, String>;
    // TODO: More attributes, possibly a generic API
}
