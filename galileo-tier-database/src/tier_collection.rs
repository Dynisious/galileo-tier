//! Defines a operations on a document collection which stores one or more tier lists.
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-06-05

use crate::{DocumentId, Document, LinkedList,};
use futures::{Future, TryFuture, FutureExt, TryFutureExt, future::MapOk,};
use std::{convert::TryInto, borrow::Borrow,};

/// A collection of documents which make up a tier list.
pub trait TierListCollection: Sized {
  /// The document type stored in the collection.
  type Document: Document;
  /// The error type returned by DB operations.
  type Error;
  /// The future type when batch fetching documents from the collection.
  type GetBatchDocuments: Future<Output = Result<Vec<Result<Self::Document, Self::Error>>, Self::Error>>;
  /// The future type when fetching a document from the collection.
  type GetDocument: Future<Output = Result<Self::Document, Self::Error>>;
  /// The future type when batch writing documents to the collection.
  type WriteBatchDocuments: Future<Output = Result<(), Vec<Result<(), Self::Error>>>>;
  /// The future type when writing a document to the collection.
  type WriteDocument: Future<Output = Result<(), Self::Error>>;

  /// Gets a batch of documents from the collection.
  /// 
  /// # Params
  /// 
  /// ids --- The identifiers of the documents in the collection.  
  fn get_documents(&self, ids: &[&DocumentId],) -> Self::GetBatchDocuments;
  /// Gets a document from the collection.
  /// 
  /// # Params
  /// 
  /// id --- The identifier of the document in the collection.  
  fn get_document(&self, id: &DocumentId,) -> Self::GetDocument;
  /// Writes documents to the collection.
  /// 
  /// # Params
  /// 
  /// documents --- The documents to write to the collection.  
  fn write_documents<T,>(&self, document: &[&T],) -> Self::WriteBatchDocuments
    where T: Borrow<Self::Document>;
  /// Writes a document to the collection.
  /// 
  /// # Params
  /// 
  /// document --- The document to write to the collection.  
  fn write_document<T,>(&self, document: &T,) -> Self::WriteDocument
    where T: Borrow<Self::Document>;
  /// Gets documents from the collection and converts them to a return type.
  /// 
  /// The type conversion is performed using [`TryInto`](doc.rust-lang.org/std/convert/trait.TryInto.html).
  /// 
  /// # Params
  /// 
  /// ids --- The identifiers of the documents in the collection.  
  fn get_items<T,>(&self, ids: &[&DocumentId],) -> MapOk<Self::GetBatchDocuments, fn(<Self::GetBatchDocuments as TryFuture>::Ok,) -> Vec<Result<T, <<<Self::GetBatchDocuments as TryFuture>::Ok as IntoIterator>::Item as TryInto<T>>::Error>>>
    where Self::GetBatchDocuments: TryFutureExt,
      <Self::GetBatchDocuments as TryFuture>::Ok: IntoIterator,
      <<Self::GetBatchDocuments as TryFuture>::Ok as IntoIterator>::Item: TryInto<T>, {
    self.get_documents(ids,)
    .map_ok(|docs,| docs.into_iter().map(TryInto::try_into,).collect(),)
  }
  /// Gets a document from the collection and converts it to a return type.
  /// 
  /// The type conversion is performed using [`TryInto`](doc.rust-lang.org/std/convert/trait.TryInto.html).
  /// 
  /// # Params
  /// 
  /// id --- The identifier of the document in the collection.  
  fn get_item<T,>(&self, id: &DocumentId,) -> MapOk<Self::GetDocument, fn(<Self::GetDocument as TryFuture>::Ok,) -> Result<T, <<Self::GetDocument as TryFuture>::Ok as TryInto<T>>::Error>>
    where Self::GetDocument: TryFutureExt,
      <Self::GetDocument as TryFuture>::Ok: TryInto<T>, {
    self.get_document(id,)
    .map_ok(|doc,| doc.try_into(),)
  }
  /// Gets a cursor at an item in the collection.
  /// 
  /// # Params
  /// 
  /// id --- The identifier of the document in the collection.  
  fn ref_cursor<'a, T,>(&'a self, id: &DocumentId,) -> MapOk<Self::GetDocument, Box<dyn 'a + FnOnce(<Self::GetDocument as TryFuture>::Ok,) -> Cursor<T, &'a Self,>>>
    where Self::GetDocument: TryFutureExt,
      <Self::GetDocument as TryFuture>::Ok: Into<T>, {
    self.get_document(id,)
    .map_ok(Box::new(move |item,| Cursor::new(self, item.into(),),),)
  }
}

/// Extended behaviour for collection types.
pub trait TierListCollectionExt: 'static + TierListCollection + Copy {
  /// Gets a cursor at an item in the collection.
  /// 
  /// # Params
  /// 
  /// id --- The identifier of the document in the collection.  
  fn get_cursor<T,>(self, id: &DocumentId,) -> MapOk<Self::GetDocument, Box<dyn FnOnce(<Self::GetDocument as TryFuture>::Ok,) -> Cursor<T, Self,>>>
    where Self::GetDocument: TryFutureExt,
      <Self::GetDocument as TryFuture>::Ok: Into<T>, {
    self.get_document(id,)
    .map_ok(Box::new(move |item,| Cursor::new(self, item.into(),),),)
  }
}

impl<'a, Coll,> TierListCollection for &'a Coll
  where Coll: TierListCollection, {
  type Document = Coll::Document;
  type Error = Coll::Error;
  type GetBatchDocuments = Coll::GetBatchDocuments;
  type GetDocument = Coll::GetDocument;
  type WriteBatchDocuments = Coll::WriteBatchDocuments;
  type WriteDocument = Coll::WriteDocument;

  #[inline]
  fn get_documents(&self, id: &[&DocumentId],) -> Self::GetBatchDocuments {
    Coll::get_documents(*self, id,)
  }
  #[inline]
  fn get_document(&self, id: &DocumentId,) -> Self::GetDocument {
    Coll::get_document(*self, id,)
  }
  #[inline]
  fn write_documents<T,>(&self, document: &[&T],) -> Self::WriteBatchDocuments
    where T: Borrow<Self::Document> {
    Coll::write_documents(*self, document,)
  }
  #[inline]
  fn write_document<T,>(&self, document: &T,) -> Self::WriteDocument
    where T: Borrow<Self::Document> {
    Coll::write_document(*self, document,)
  }
}

/// A view into a collection.
#[derive(PartialEq, Eq, Clone, Copy, Debug,)]
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
  /// Breaks the cursor into its component parts.
  #[inline]
  pub fn into_parts(self,) -> (Coll, T,) { (self.collection, self.item,) }
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
  pub fn move_next(self,) -> Result<impl Future<Output = Result<Self, (Self, Coll::Error,)>>, Self>
    where Coll::GetDocument: FutureExt,
      Coll::Document: Into<T>, {
    //Get the id of the next node.
    match self.item.get_next_id() {
      //There is a next node.
      Some(next_id) => Ok(
        //Get the next node.
        self.collection.get_document(next_id,)
        .map(move |res,| match res {
          Ok(item) => Ok(Self { item: item.into(), ..self }),
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
  pub fn move_previous(self,) -> Result<impl Future<Output = Result<Self, (Self, Coll::Error,)>>, Self>
    where Coll::GetDocument: FutureExt,
      Coll::Document: Into<T>, {
    //Get the id of the next node.
    match self.item.get_previous_id() {
      //There is a next node.
      Some(previous_id) => Ok(
        //Get the next node.
        self.collection.get_document(previous_id,)
        .map(move |res,| match res {
          Ok(item) => Ok(Self { item: item.into(), ..self }),
          Err(e) => Err((self, e,))
        },)
      ),
      //There is no node.
      None => Err(self)
    }
  }
  /// Gets a `Cursor` to the next node in the linked list.
  pub async fn ref_next(&self,) -> Result<Option<Cursor<T, &Coll,>>, <Coll::GetDocument as TryFuture>::Error>
    where Coll::GetDocument: TryFutureExt,
      <Coll::GetDocument as TryFuture>::Ok: Into<T>, {
    //Get the next id.
    match self.item.get_next_id() {
      //Get the cursor.
      Some(next_id) => self.collection.ref_cursor(next_id,).await.map(Some),
      None => Ok(None),
    }
  }
  /// Gets a `Cursor` to the previous node in the linked list.
  pub async fn ref_previous(&self,) -> Result<Option<Cursor<T, &Coll,>>, <Coll::GetDocument as TryFuture>::Error>
    where Coll::GetDocument: TryFutureExt,
      <Coll::GetDocument as TryFuture>::Ok: Into<T>, {
    //Get the previous id.
    match self.item.get_previous_id().cloned() {
      //Get the cursor.
      Some(previous_id) => self.collection.ref_cursor(&previous_id,).await.map(Some),
      None => Ok(None),
    }
  }
}

impl<T, Coll,> Cursor<T, Coll,>
  where T: LinkedList,
    Coll: TierListCollection + Copy, {
  /// Gets a `Cursor` to the next node in the linked list.
  pub async fn get_next(&self,) -> Result<Option<Cursor<T, Coll,>>, <Coll::GetDocument as TryFuture>::Error>
    where Coll::GetDocument: TryFutureExt,
      <Coll::GetDocument as TryFuture>::Ok: Into<T>, {
    //Get the next id.
    match self.item.get_next_id() {
      //Get the cursor.
      Some(next_id) => self.collection.ref_cursor(next_id,).await
        .map(|cursor,| Some(cursor.copied_coll()),),
      None => Ok(None),
    }
  }
  /// Gets a `Cursor` to the previous node in the linked list.
  pub async fn get_previous(&self,) -> Result<Option<Cursor<T, Coll,>>, <Coll::GetDocument as TryFuture>::Error>
    where Coll::GetDocument: TryFutureExt,
      <Coll::GetDocument as TryFuture>::Ok: Into<T>, {
    //Get the previous id.
    match self.item.get_previous_id().cloned() {
      //Get the cursor.
      Some(previous_id) => self.collection.ref_cursor(&previous_id,).await
        .map(|cursor,| Some(cursor.copied_coll()),),
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

#[cfg(test,)]
mod tests {
  use super::*;
  use futures::{future, Future,};
  use std::{
    collections::HashMap,
    cell::RefCell,
    pin::Pin,
    rc::Rc,
  };

  #[derive(PartialEq, Eq, Clone, Copy, Debug,)]
  struct Doc {
    id: DocumentId,
    next: Option<DocumentId>,
    prev: Option<DocumentId>,
  }

  impl Document for Doc {
    #[inline]
    fn get_id(&self,) -> &DocumentId { &self.id }
  }

  impl LinkedList for Doc {
    #[inline]
    fn get_next_id(&self,) -> Option<&DocumentId> {
      self.next.as_ref()
    }
    #[inline]
    fn get_previous_id(&self,) -> Option<&DocumentId> {
      self.prev.as_ref()
    }
  }

  impl TierListCollection for Rc<RefCell<HashMap<DocumentId, Doc>>> {
    type Document = Doc;
    type Error = !;
    type GetDocument = Pin<Box<dyn Future<Output = Result<Self::Document, Self::Error>>>>;
    type GetBatchDocuments = Pin<Box<dyn Future<Output = Result<Vec<Result<Self::Document, Self::Error>>, Self::Error>>>>;
    type WriteDocument = Pin<Box<dyn Future<Output = Result<(), Self::Error>>>>;
    type WriteBatchDocuments = Pin<Box<dyn Future<Output = Result<(), Vec<Result<(), Self::Error>>>>>>;

    fn get_documents(&self, ids: &[&DocumentId],) -> Self::GetBatchDocuments {
      let coll = self.clone();
      let ids = ids.into_iter()
        .map(|&&id,| id,)
        .collect::<Vec<_>>();

      Box::pin(async move {
        Ok(future::join_all(
          ids.into_iter().map(|id,| coll.get_document(&id,),),
        ).await)
      },)
    }
    fn get_document(&self, id: &DocumentId,) -> Self::GetDocument {
      use std::task::Poll;

      let coll = self.clone();
      let id = *id;

      Box::pin(future::poll_fn(move |_,| {
        match coll.try_borrow() {
          Ok(coll) => Poll::Ready(Ok(coll[&id])),
          Err(_) => Poll::Pending,
        }
      },),)
    }
    fn write_documents<T,>(&self, documents: &[&T],) -> Self::WriteBatchDocuments
      where T: Borrow<Self::Document>, {
      let coll = self.clone();
      let docs = documents.into_iter()
        .map(|&doc,| *doc.borrow(),)
        .collect::<Vec<_>>();

      Box::pin(async move {
        let mut all_succeeded = true;
        let mut results = Vec::with_capacity(docs.len(),);
        let docs = docs.into_iter()
          .map(|doc,| coll.write_document(&doc,),);

        for res in docs {
          let res = res.await;

          all_succeeded = all_succeeded && res.is_ok();
          results.push(res,);
        }

        if all_succeeded { Ok(()) }
        else { Err(results) }
      },)
    }
    fn write_document<T,>(&self, document: &T,) -> Self::WriteDocument
      where T: Borrow<Self::Document>, {
      use std::task::Poll;

      let coll = self.clone();
      let document = *document.borrow();

      Box::pin(future::poll_fn(move |_,| {
        match coll.try_borrow_mut() {
          Ok(mut coll) => {
            coll.insert(*document.get_id(), document,);

            Poll::Ready(Ok(()))
          },
          Err(_) => Poll::Pending,
        }
      },),)
    }
  }

  #[test]
  fn test_cursor() {
    use futures::{executor::LocalPool, task::LocalSpawnExt,};
    
    let coll = Rc::new(RefCell::new(HashMap::new(),),);
    let id1 = [1u8; 20];
    let id2 = [2u8; 20];
    let id3 = [3u8; 20];
    let id4 = [4u8; 20];
    let doc1 = Doc {
      id: id1,
      prev: None,
      next: None,
    };
    let doc2 = Doc {
      id: id2,
      prev: None,
      next: Some(id3),
    };
    let doc3 = Doc {
      id: id3,
      prev: Some(id2),
      next: Some(id4),
    };
    let doc4 = Doc {
      id: id4,
      prev: Some(id3),
      next: None,
    };
    let mut pool = LocalPool::new();
    let fut = Box::pin(async move {
      coll.write_document(&doc1,).await
        .expect("Error writing document");
      
      let docs = coll.write_documents(&[&doc2, &doc3, &doc4,],).await;
    
      if let Err(res) = docs {
        for (i, res) in res.into_iter().enumerate().filter(|(_, res,),| res.is_err(),) {
          res.expect(&format!("Error writing id{}", i,),)
        }
      }

      let cursor = coll.ref_cursor::<Doc>(&id3,).await.unwrap();
      assert_eq!(cursor.get_item(), &doc3, "Error Cursor at wrong document",);

      let next_cursor = cursor.get_next().await.unwrap()
        .expect("No next Cursor");
      assert_eq!(next_cursor.get_item(), &doc4, "Error next Cursor at wrong document",);
      assert!(
        next_cursor.get_next().await.unwrap().is_none(),
        "Error next Cursor is not the last document",
      );

      let other_cursor = next_cursor.move_previous()
        .expect("Error next Cursor cannot move back").await
        .expect("Error moving next Cursor back");
      assert_eq!(other_cursor, cursor, "Error other Cursor at wrong document",);

      let prev_cursor = cursor.get_previous().await.unwrap()
        .expect("No previous Cursor");
      assert_eq!(prev_cursor.get_item(), &doc2, "Error previous Cursor at wrong document",);
      assert!(
        prev_cursor.get_previous().await.unwrap().is_none(),
        "Error previous Cursor is not the first document",
      );
    },);
    
    pool.spawner()
      .spawn_local(fut,)
      .expect("Error spawning task");
    pool.run();
  }
}
