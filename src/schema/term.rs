use std::fmt;

use common;
use byteorder::{BigEndian, ByteOrder};
use super::Field;
use std::str;


/// Size (in bytes) of the buffer of a int field.
const INT_TERM_LEN: usize = 4 + 8;

/// Term represents the value that the token can take.
///
/// It actually wraps a `Vec<u8>`.
#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Term(Vec<u8>);

/// Extract `field` from Term.
pub(crate) fn extract_field_from_term_bytes(term_bytes: &[u8]) -> Field {
    Field(BigEndian::read_u32(&term_bytes[..4]))
}

impl Term {
    /// Returns the field.
    pub fn field(&self,) -> Field {
        extract_field_from_term_bytes(&self.0)
    }

    /// Returns the field.
    pub fn set_field(&mut self, field: Field) {
        if self.0.len() < 4 {
            self.0.resize(4, 0u8);
        }
        BigEndian::write_u32(&mut self.0[0..4], field.0);
    }

    /// Builds a term given a field, and a u64-value
    ///
    /// Assuming the term has a field id of 1, and a u64 value of 3234,
    /// the Term will have 8 bytes.
    ///
    /// The first four byte are dedicated to storing the field id as a u64.
    /// The 4 following bytes are encoding the u64 value.
    pub fn from_field_u64(field: Field, val: u64) -> Term {
        let mut term = Term(vec![0u8; INT_TERM_LEN]);
        term.set_field(field);
        term.set_u64(val);
        term
    }

    /// Sets a u64 value in the term.
    ///
    /// U64 are serialized using (8-byte) BigEndian
    /// representation.
    /// The use of BigEndian has the benefit of preserving
    /// the natural order of the values.
    pub fn set_u64(&mut self, val: u64) {
        self.0.resize(INT_TERM_LEN, 0u8);
        BigEndian::write_u64(&mut self.0[4..], val);
    }

    /// Builds a term given a field, and a u64-value
    ///
    /// Assuming the term has a field id of 1, and a u64 value of 3234,
    /// the Term will have 8 bytes.
    ///
    /// The first four byte are dedicated to storing the field id as a u64.
    /// The 4 following bytes are encoding the u64 value.
    pub fn from_field_i64(field: Field, val: i64) -> Term {
        let val_u64: u64 = common::i64_to_u64(val);
        Term::from_field_u64(field, val_u64)
    }

    /// Builds a term given a field, and a string value
    ///
    /// Assuming the term has a field id of 2, and a text value of "abc",
    /// the Term will have 4 bytes.
    /// The first byte is 2, and the three following bytes are the utf-8
    /// representation of "abc".
    pub fn from_field_text(field: Field, text: &str) -> Term {
        let buffer = Vec::with_capacity(4 + text.len());
        let mut term = Term(buffer);
        term.set_field(field);
        term.set_text(text);
        term
    }

    /// Creates a new Term with an empty buffer,
    /// but with a given capacity.
    ///
    /// It is declared unsafe, as the term content
    /// is not initialized, and a call to `.field()`
    /// would panic.
    pub(crate) unsafe fn with_capacity(num_bytes: usize) -> Term {
        Term(Vec::with_capacity(num_bytes))
    }

    /// Assume the term is a u64 field.
    ///
    /// Panics if the term is not a u64 field.
    pub fn get_u64(&self) -> u64 {
        BigEndian::read_u64(&self.0[4..])
    }

    /// Builds a term from its byte representation.
    ///
    /// If you want to build a field for a given `str`,
    /// you want to use `from_field_text`.
    pub(crate) fn from_bytes(data: &[u8]) -> Term {
        Term(Vec::from(data))
    }

    /// Returns the serialized value of the term.
    /// (this does not include the field.)
    ///
    /// If the term is a string, its value is utf-8 encoded.
    /// If the term is a u64, its value is encoded according
    /// to `byteorder::LittleEndian`.
    pub fn value(&self) -> &[u8] {
        &self.0[4..]
    }

    /// Returns the text associated with the term.
    ///
    /// # Panics
    /// If the value is not valid utf-8. This may happen
    /// if the index is corrupted or if you try to
    /// call this method on a non-string type.
    pub fn text(&self) -> &str {
        str::from_utf8(self.value()).expect("Term does not contain valid utf-8.")
    }

    /// Set the texts only, keeping the field untouched.
    pub fn set_text(&mut self, text: &str) {
        self.0.resize(4, 0u8);
        self.0.extend(text.as_bytes());
    }

    /// Returns the underlying `&[u8]`
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Term {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Term({:?})", &self.0[..])
    }
}


#[cfg(test)]
mod tests {

    use schema::*;

    #[test]
    pub fn test_term() {
        let mut schema_builder = SchemaBuilder::default();
        schema_builder.add_text_field("text", STRING);
        let title_field = schema_builder.add_text_field("title", STRING);
        let count_field = schema_builder.add_text_field("count", STRING);
        {
            let term = Term::from_field_text(title_field, "test");
            assert_eq!(term.field(), title_field);
            assert_eq!(&term.as_slice()[0..4], &[0u8, 0u8, 0u8, 1u8]);
            assert_eq!(&term.as_slice()[4..], "test".as_bytes());
        }
        {
            let term = Term::from_field_u64(count_field, 983u64);
            assert_eq!(term.field(), count_field);
            assert_eq!(&term.as_slice()[0..4], &[0u8, 0u8, 0u8, 2u8]);
            assert_eq!(term.as_slice().len(), 4 + 8);
            assert_eq!(term.as_slice()[4], 0u8);
            assert_eq!(term.as_slice()[5], 0u8);
            assert_eq!(term.as_slice()[6], 0u8);
            assert_eq!(term.as_slice()[7], 0u8);
            assert_eq!(term.as_slice()[8], 0u8);
            assert_eq!(term.as_slice()[9], 0u8);
            assert_eq!(term.as_slice()[10], (933u64 / 256u64) as u8);
            assert_eq!(term.as_slice()[11], (983u64 % 256u64) as u8);
        }

    }
}
