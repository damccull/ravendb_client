
/// Implements Unit of Work for accessing the RavenDB server.
pub struct DocumentSession;

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

    /// Saves all the pending changes in the session to the server.
    pub async fn save_changes() -> Result<(), DocumentSessionError> {
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
