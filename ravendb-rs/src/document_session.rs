use std::collections::HashMap;

use crate::{document_conventions::DocumentConventions, DocumentStore};

mod session_impls;

/// Implements Unit of Work for accessing the RavenDB server.
pub struct DocumentSession {
    document_conventions: DocumentConventions,

    // -- START: From InMemoryDocumentSessionOperations.cs
    async_task_counter: i64, //TODO: See if this can be u32 or u64. Probably never negative.
    max_docs_count_on_cached_renew_session: i32,
    request_executor: RequestExecutor,
    operation_executor: OperationExecutor,
    release_operation_context: PlaceholderType, //TODO: This is "IDisposable" in C#, meaning we need to ensure destructor runs?
    context: JsonOperationContext,
    pending_lazy_operations: Vec<Box<dyn LazyOperation + Send>>,
    on_evaluate_lazy: HashMap<Box<dyn LazyOperation + Send>, Action<PlaceholderType>>, // PlaceHolderType here is 'object' in C#, meaning any type at all. How do I do that here?
    instances_counter: i32,
    hash: i32,
    generate_document_ids_on_store: bool,
    session_info: SessionInfo,
    // -- END: From InMemoryDocumentSessionOperations.cs

    // -- START: From InMemoryDocumentSessionOperations.Patch.cs
    // -- END: From InMemoryDocumentSessionOperations.Patch.cs
}

impl DocumentSession {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        todo!();
        // Self {
        //     _document_conventions: DocumentConventions,
        // }
    }
}

// From DocumentSession.cs
impl DocumentSession {
    /// Saves all the pending changes in the session to the server.
    pub async fn save_changes() -> Result<(), DocumentSessionError> {
        todo!();
    }

    /// Check if a document exists without loading it.
    pub async fn exists() -> bool {
        todo!()
    }

    /// Refreshes the specified entity from the RavenDB server.
    pub async fn refresh<T>(entity: T) {
        todo!()
    }

    /// Generates the document ID.
    // TODO: Figure out if this can be done generically
    // The C# code uses a type of "object" here to accept any type at all
    async fn generate_id<T>(entity: T) -> String {
        todo!()
    }

    /// Executes all pending lazy operations.
    pub async fn execute_all_pending_lazy_operations() -> ResponseTimeInformation {
        todo!()
    }

    fn execute_lazy_operations_single_step(
        response_time_information: ResponseTimeInformation,
        requests: Vec<GetRequest>,
        stopwatch: Stopwatch,
    ) {
        todo!()
    }
}

// From IDocumentSession.cs
impl DocumentSession {
    /// Marks the specified entity for deletion.
    ///
    /// Does not delete it immediately but only when
    /// [`save_changes()`](DocumentSession::save_changes) is called.
    ///
    /// Takes a reference to the entity itself.
    pub async fn delete_entity<T>(_entity: &T) -> Result<(), DocumentSessionError> {
        todo!();
    }

    /// Marks the entity specified by `id` for deletion.
    ///
    /// Does not delete it immediately but only when
    /// [`save_changes()`](DocumentSession::save_changes) is called.  
    ///
    /// WARNING: This method will not call beforeDelete listener!
    pub async fn delete_by_id(_id: &str) -> Result<(), DocumentSessionError> {
        todo!();
    }

    /// Marks the entity specified by `id` for deletion assuming the `expected_change_vector`
    /// is correct.
    ///
    /// Does not delete it immediately but only when
    /// [`save_changes()`](DocumentSession::save_changes) is called.
    ///
    /// NOTE: This will not succeed if the expect change vector doesn't match the resulting
    /// change vector.
    ///
    /// WARNING: This method will not call beforeDelete listener!
    pub async fn delete_by_id_and_change_vector(
        _id: &str,
        _expected_change_vector: &str,
    ) -> Result<(), DocumentSessionError> {
        todo!();
    }

    /// Stores the entity in the session and extracts the `id` from the entity or else generates one
    /// according to the conventions of the [`DocumentStore`] if the entity doesn't supply one.
    ///
    /// Will not save to the database immediately, but only when
    /// [`save_changes()`](DocumentSession::save_changes) is called.
    ///
    /// NOTE: If the `id` is not available during extraction, this will force a concurrency check.
    pub async fn store_entity<T>(_entity: T) -> Result<(), DocumentSessionError> {
        todo!();
    }

    /// Stores the entity in the session with the supplied `id` and forces a concurrency check with
    /// the supplied change vector.
    ///
    /// Will not save to the database immediately, but only when
    /// [`save_changes()`](DocumentSession::save_changes) is called.
    pub async fn store_entity_with_change_vector<T>(
        _entity: T,
        _change_vector: &str,
        _id: &str,
    ) -> Result<(), DocumentSessionError> {
        todo!();
    }

    /// Stores the entity in the session with the supplied `id`.
    ///
    /// Will not save to the database immediately, but only when
    /// [`save_changes()`](DocumentSession::save_changes) is called.
    ///
    /// NOTE: This will overwrite any entity with the supplied `id` that already exists in the session.
    pub async fn store_entity_with_id<T>(
        _entity: T,
        _id: &str,
    ) -> Result<(), DocumentSessionError> {
        todo!();
    }
}

pub struct DocumentSessionError;

// Temporary placeholder structs
struct Entity;
struct RequestExecutor;
struct OperationExecutor;
trait LazyOperation {}
struct SessionInfo;
struct BatchOptions;
struct PlaceholderType;
struct DocumentsById;
struct DocumentInfo;
struct IdTypeAndName;
struct CommandData;
struct GenerateEntityIdOnTheClient;
struct EntityToJson;
pub struct ResponseTimeInformation;
struct Stopwatch;
struct GetRequest;
struct JsonOperationContext;
struct Action<T> {
    phantom: T,
}
