#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Noop")]
    Noop,
}
