/*!
ravendb_client is a client library for the RavenDB document database.
It aims to compete with the existing libraries officially offered, but
using more rusty ways of doing things.

This library requires tokio and async, and uses the actor pattern to maintain
a single instance of the [`DocumentStore`] per cluster, as recommended by the
official libraries. This is designed to keep resource usage in your app to a minimum.

A [`DocumentSession`] can be requested from the [`DocumentStore`] to interact with the
database. It is considered a unit of work and all changes to a [`DocumentSession`] will
succeed or fail together.

# Example
// ```rust
// use ravendb_client::DocumentStore;

// let document_store = DocumentStore::new();
// let session = document_store.open_session();

// //...update entities here...

// session.save_changes();
// ```

When `session` is dropped, it'll close any open handles appropriately on its own.
*/

mod document_session;
mod document_store;

pub mod events;

pub use document_session::*;
pub use document_store::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
