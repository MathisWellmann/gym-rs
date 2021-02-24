/// Defines the type of action to take in the environment
pub enum ActionType {
    /// A discrete action
    Discrete(u8),

    /// A continuous action
    Continuous(Vec<f64>),
}
