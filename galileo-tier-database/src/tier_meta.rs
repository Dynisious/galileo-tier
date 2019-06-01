//! Defines a representation of a tier making up a tier list in a document collection.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-06-02

use crate::{DocumentId, Document, LinkedList,};
use std::num::NonZeroU64;

/// Metadata for a collection of `Card`s making up a tier.
#[derive(PartialEq, Eq,)]
pub struct TierMeta {
  /// The Id of this `TierMeta`.
  pub id: DocumentId,
  /// The length and ends of the doubly linked list of `Card`s making up the tier.
  ends: (Option<NonZeroU64>, DocumentId, DocumentId,),
  /// The Id of the previous tier.
  pub previous_tier: Option<DocumentId>,
  /// The Id of the next tier.
  pub next_tier: Option<DocumentId>,
}

impl Document for TierMeta {
  #[inline]
  fn get_id(&self,) -> &DocumentId { &self.id }
}

impl LinkedList for TierMeta {
  #[inline]
  fn get_previous_id(&self,) -> Option<&DocumentId> { self.previous_tier.as_ref() }
  #[inline]
  fn get_next_id(&self,) -> Option<&DocumentId> { self.next_tier.as_ref() }
}
