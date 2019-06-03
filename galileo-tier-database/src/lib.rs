//! Defines the types needed for a bare bones implementation of a `galileo tier list`.
//! 
//! The concept of a `galileo tier list` is a publicly viewable tier list where items
//! move between tiers based on upvotes and downvotes.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-06-02

#![deny(missing_docs,)]
#![feature(async_await, await_macro, associated_type_defaults,
  const_fn, never_type, gen_future, try_trait, generator_trait,
  generators,
)]

mod card;
mod tier_meta;
mod tier_collection;

pub use self::{card::*, tier_meta::*, tier_collection::*,};

/// The identifier for a document.
pub type DocumentId = [u8; 20];

/// A trait which defines the common elements of database documents.
pub trait Document {
  /// Gets the `DocumentId` of this document.
  fn get_id(&self,) -> &DocumentId;
}

/// A trait for database documents which are also nodes in a doubly linked list.
pub trait LinkedList: Document {
  /// Gets the identifier of previous document.
  fn get_previous_id(&self,) -> Option<&DocumentId>;
  /// Gets the identifier of next document.
  fn get_next_id(&self,) -> Option<&DocumentId>;
  /// Returns `true` if this is the front of the linked list.
  #[inline]
  fn is_front(&self,) -> bool { self.get_previous_id().is_none() }
  /// Returns `true` if this is the back of the linked list.
  #[inline]
  fn is_back(&self,) -> bool { self.get_next_id().is_none() }
}
