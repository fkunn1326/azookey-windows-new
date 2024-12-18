#[derive(Debug, PartialEq)]
pub enum ClientAction {
    StartComposition,
    EndComposition,

    AppendText(String),
    RemoveText,
}