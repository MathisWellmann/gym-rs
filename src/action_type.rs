pub enum ActionType {
    Discrete(u8),
    Continuous(Vec<f64>),
}