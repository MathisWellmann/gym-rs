use crate::GifRender;

/// The trait which defines the needed methods an environment needs to provide
pub trait GymEnv {
    type ActionType;

    /**
    Run one timestep of the environment's dynamics. When end of episode is reached,
    you you are responsible for calling 'reset()' to reset this environment's state

    Accepts an action and returns a tuple (observation, reward, done, info)

    Returns:
        observation: agent's observation of the current environment
        reward: amount of reward returned after previous action
        done: whether the episode has ended
        info: optional information string
    **/
    fn step(&mut self, action: Self::ActionType) -> (Vec<f64>, f64, bool, Option<String>);

    /// Reset the environment to an initial state
    /// This function should not reset reset the environment's random number generator(s)
    /// Returns the environments initial state
    fn reset(&mut self) -> Vec<f64>;

    /// Render the environment
    fn render(&self, viewer: &mut GifRender);

    /// Set the seed for this environments random number generator
    fn seed(&mut self, seed: u64);
}
