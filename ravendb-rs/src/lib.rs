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
