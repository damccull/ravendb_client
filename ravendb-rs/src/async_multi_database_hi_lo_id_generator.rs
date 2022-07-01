use dyn_clone::DynClone;
use std::fmt::Debug;

pub trait AsyncDocumentIdGenerator: Debug + DynClone + Send {}

dyn_clone::clone_trait_object!(AsyncDocumentIdGenerator);

#[derive(Clone, Debug, Default)]
pub struct AsyncMultiDatabaseHiLoIdGenerator;
impl AsyncDocumentIdGenerator for AsyncMultiDatabaseHiLoIdGenerator {}
