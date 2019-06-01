//! Defines a operations on a document collection which stores one or more tier lists.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-06-02

use crate::{DocumentId, Document, LinkedList,};
use futures::{Future, TryFuture, FutureExt, TryFutureExt, future::{Map, MapOk,},};
use std::convert::TryInto;

/// A collection of documents which make up a tier list.
pub trait TierListCollection: Sized {
  /// The document type stored in the collection.
  type Document: Document;
  /// The error type returned by DB operations.
  type Error;
  /// The future type when fetching documents from the collection.
  type Future: Future<Output = Result<Self::Document, Self::Error>>;
  /// The future type when writing documents to the collection.
  type WriteFuture: Future<Output = Result<(), Self::Error>>;

  /// Gets a document from the collection by its unique identifier.
  /// 
  /// # Params
  /// 
  /// id --- The identifier of the document in the collection.  
  fn get_document(&self, id: &DocumentId,) -> Self::Future;
  /// Writes a document to the collection.
  /// 
  /// # Params
  /// 
  /// document --- The document to write to the collection.  
  fn write_document<T,>(&self, document: &T,) -> Self::WriteFuture
    where T: AsRef<Self::Document>;
  /// Gets a document from the collection and converts it to a type.
  /// 
  /// The conversion is performed using [`TryInto`](doc.rust-lang.org/std/convert/trait.TryInto.html).
  /// 
  /// # Params
  /// 
  /// id --- The identifier of the document in the collection.  
  #[inline]
  fn get_item<T,>(&self, id: &DocumentId,) -> Map<Self::Future, fn(<Self::Future as Future>::Output,) -> Result<T, Self::Error>>
    where Self::Document: TryInto<T>,
      Self::Future: FutureExt,
      Self::Error: From<<Self::Document as TryInto<T>>::Error>, {
    self.get_document(id,)
    .map(|doc,| doc?.try_into().map_err(From::from,),)
  }
  /// Writes an item to the collection.
  /// 
  /// The conversion is performed using [`Into`](doc.rust-lang.org/std/convert/trait.Into.html).
  /// 
  /// # Params
  /// 
  /// document --- The document to write to the collection.  
  fn write_item<T,>(&self, document: T,) -> Self::WriteFuture
    where T: Into<Self::Document>,
      Self::Document: AsRef<Self::Document>, {
    self.write_document(&document.into(),)
  }
  /// Gets a cursor at an item in the collection .
  /// 
  /// # Params
  /// 
  /// id --- The identifier of the document in the collection.  
  fn get_cursor<'a, T,>(&'a self, id: &DocumentId,) -> MapOk<Map<Self::Future, fn(<Self::Future as Future>::Output,) -> Result<T, Self::Error>>, Box<dyn 'a + FnOnce(T,) -> Cursor<T, &'a Self,>>>
    where Self::Document: TryInto<T>,
      Self::Future: TryFutureExt,
      Self::Error: From<<Self::Document as TryInto<T>>::Error>, {
    self.get_item(id,)
    .map_ok(Box::new(move |item,| Cursor::new(self, item,),),)
  }
}

impl<'a, Coll,> TierListCollection for &'a Coll
  where Coll: TierListCollection, {
  type Document = Coll::Document;
  type Error = Coll::Error;
  type Future = Coll::Future;
  type WriteFuture = Coll::WriteFuture;

  #[inline]
  fn get_document(&self, id: &DocumentId,) -> Self::Future { Coll::get_document(*self, id,) }
  #[inline]
  fn write_document<T,>(&self, document: &T,) -> Self::WriteFuture
    where T: AsRef<Self::Document>, {
    Coll::write_document(*self, document,)
  }
}

/// A view into a collection.
#[derive(PartialEq, Eq, Clone, Copy,)]
pub struct Cursor<T, Coll,>
  where Coll: TierListCollection, {
  /// The `TierCollection` to get items from.
  collection: Coll,
  /// The item at this `Cursor`.
  item: T,
}

impl<T, Coll,> Cursor<T, Coll,>
  where Coll: TierListCollection, {
  #[inline]
  const fn new(collection: Coll, item: T,) -> Self {
    Self { collection, item, }
  }
  /// Maps the value stored in this `Cursor`.
  #[inline]
  pub fn map<U, F,>(self, map: F,) -> Cursor<U, Coll,>
    where F: FnOnce(T,) -> U, {
    Cursor::new(self.collection, map(self.item,),)
  }
  /// Gets the collection used by this `Cursor`.
  #[inline]
  pub const fn get_collection(&self,) -> &Coll { &self.collection }
  /// Gets the item at this cursor.
  #[inline]
  pub const fn get_item(&self,) -> &T { &self.item }
  /// References the value inside this `Cursor`.
  #[inline]
  pub const fn as_ref(&self,) -> Cursor<&T, &Coll,> {
    Cursor::new(self.get_collection(), self.get_item(),)
  }
}

impl<T, Coll,> Cursor<T, Coll,>
  where T: LinkedList,
    Coll: TierListCollection, {
  /// Moves this `Cursor` to the next node in the linked list.
  /// 
  /// If there is no next node this `Cursor` is returned unchanged as `Error(self)`.
  /// 
  /// If there was an error getting the next node this `Cursor` is returned unchanged as
  /// `Err((self, error))`.
  pub fn move_next(self,) -> Result<impl TryFuture<Ok = Self, Error = (Self, Coll::Error,)>, Self>
    where Coll::Document: TryInto<T>,
      Coll::Future: FutureExt,
      Coll::Error: From<<Coll::Document as TryInto<T>>::Error>, {
    //Get the id of the next node.
    match self.item.get_next_id().cloned() {
      //There is a next node.
      Some(next_id) => Ok(
        //Get the next node.
        self.collection.get_item(&next_id,)
        .map(move |res,| match res {
          Ok(item) => Ok(Self { item, ..self }),
          Err(e) => Err((self, e,))
        },)
      ),
      //There is no node.
      None => Err(self)
    }
  }
  /// Moves this `Cursor` to the previous node in the linked list.
  /// 
  /// If there is no previous node this `Cursor` is returned unchanged.
  /// 
  /// If there was an error getting the previous node this `Cursor` is returned unchanged.
  pub fn move_previous(self,) -> Result<impl TryFuture<Ok = Self, Error = (Self, Coll::Error,)>, Self>
    where Coll::Document: TryInto<T>,
      Coll::Future: FutureExt,
      Coll::Error: From<<Coll::Document as TryInto<T>>::Error>, {
    //Get the id of the next node.
    match self.item.get_previous_id().cloned() {
      //There is a next node.
      Some(previous_id) => Ok(
        //Get the next node.
        self.collection.get_item(&previous_id,)
        .map(move |res,| match res {
          Ok(item) => Ok(Self { item, ..self }),
          Err(e) => Err((self, e,))
        },)
      ),
      //There is no node.
      None => Err(self)
    }
  }
  /// Gets a `Cursor` to the next node in the linked list.
  pub async fn get_next(&self,) -> Result<Option<Cursor<T, &Coll,>>, Coll::Error>
    where Coll::Document: TryInto<T>,
      Coll::Future: FutureExt + TryFutureExt,
      Coll::Error: From<<Coll::Document as TryInto<T>>::Error>, {
    //Get the next id.
    match self.item.get_next_id().cloned() {
      //Get the cursor.
      Some(next_id) => self.collection.get_cursor(&next_id,).await.map(Some),
      None => Ok(None),
    }
  }
  /// Gets a `Cursor` to the previous node in the linked list.
  pub async fn get_previous(&self,) -> Result<Option<Cursor<T, &Coll,>>, Coll::Error>
    where Coll::Document: TryInto<T>,
      Coll::Future: FutureExt + TryFutureExt,
      Coll::Error: From<<Coll::Document as TryInto<T>>::Error>, {
    //Get the previous id.
    match self.item.get_previous_id().cloned() {
      //Get the cursor.
      Some(previous_id) => self.collection.get_cursor(&previous_id,).await.map(Some),
      None => Ok(None),
    }
  }
}

impl<'t, T, Coll,> Cursor<&'t T, Coll,>
  where T: Clone,
    Coll: TierListCollection, {
  /// Clones the item stored by this Cursor.
  #[inline]
  pub fn cloned(self,) -> Cursor<T, Coll,> {
    Cursor::new(self.collection, self.item.clone(),)
  }
}

impl<'t, T, Coll,> Cursor<&'t T, Coll,>
  where T: Copy,
    Coll: TierListCollection, {
  /// Copies the item stored by this Cursor.
  #[inline]
  pub fn copied(self,) -> Cursor<T, Coll,> {
    Cursor::new(self.collection, *self.item,)
  }
}

impl<'coll, T, Coll,> Cursor<T, &'coll Coll,>
  where Coll: TierListCollection + Clone, {
  /// Clones the collection interface used by this Cursor.
  #[inline]
  pub fn cloned_coll(self,) -> Cursor<T, Coll,> {
    Cursor::new(self.collection.clone(), self.item,)
  }
}

impl<'coll, T, Coll,> Cursor<T, &'coll Coll,>
  where Coll: TierListCollection + Copy, {
  /// Copies the item stored by this Cursor.
  #[inline]
  pub fn copied_coll(self,) -> Cursor<T, Coll,> {
    Cursor::new(*self.collection, self.item,)
  }
}
