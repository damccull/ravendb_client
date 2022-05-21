pub enum RequestEvents {
    BeforeRequest,
    FailedRequest,
    SucceedRequest,
}

pub enum CrudEvents {
    AfterSaveChanges,
    BeforeDelete,
    BeforeQuery,
    BeforeStore,
}

pub enum ConversionEvents {
    AfterConversionToDocument,
    AfterConversionToEntity,
    BeforeConversionToDocument,
    BeforeConversionToEntity,
}

pub enum SessionEvents {
    SessionClosing,
    SessionCreated,
    TopologyUpdate,
}
