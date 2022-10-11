#[derive(Debug)]
pub enum SessionEvent {
    Enter,
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    Die,
}
