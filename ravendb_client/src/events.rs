#[derive(Clone, Debug)]
pub enum RequestEvents {
    BeforeRequest,
    FailedRequest,
    SucceedRequest,
}

#[derive(Clone, Debug)]
pub enum CrudEvents {
    AfterSaveChanges,
    BeforeDelete,
    BeforeQuery,
    BeforeStore,
}

#[derive(Clone, Debug)]
pub enum ConversionEvents {
    AfterConversionToDocument,
    AfterConversionToEntity,
    BeforeConversionToDocument,
    BeforeConversionToEntity,
}

#[derive(Clone, Debug)]
pub enum SessionEvents {
    SessionClosing,
    SessionCreated,
    TopologyUpdate,
}
