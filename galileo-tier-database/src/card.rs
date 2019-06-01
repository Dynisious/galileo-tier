//! Defines a representation of a card making up a tier in a document collection.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-06-02

use crate::{DocumentId, Document, LinkedList,};

/// Defines an individual `Card`.
#[derive(PartialEq, Eq, Clone, Debug,)]
pub struct Card {
  /// The identifier of this `Card`.
  pub id: DocumentId,
  /// The display name of this `Card`.
  pub name: String,
  /// The description of this `Card`.
  pub description: String,
  /// The up votes on this `Card`.
  pub up_votes: u64,
  /// The down votes on this `Card`.
  pub down_votes: u64,
  /// The bias which drags this `Card` down in addition to down votes.
  pub bias: u64,
  /// The Id of the previous `Card` in the current tier.
  pub previous_card: Option<DocumentId>,
  /// The Id of the next `Card` in the current tier.
  pub next_card: Option<DocumentId>,
}

impl Document for Card {
  #[inline]
  fn get_id(&self,) -> &DocumentId { &self.id }
}

impl LinkedList for Card {
  #[inline]
  fn get_previous_id(&self,) -> Option<&DocumentId> { self.previous_card.as_ref() }
  #[inline]
  fn get_next_id(&self,) -> Option<&DocumentId> { self.next_card.as_ref() }
}
