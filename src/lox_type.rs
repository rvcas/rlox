#[derive(Debug, Clone)]
pub enum LoxType {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}
