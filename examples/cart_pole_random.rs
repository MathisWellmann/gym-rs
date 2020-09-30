/*
Cart Pole Environment using random actions
*/

use gym_rs::{CartPoleEnv, GymEnv, ActionType};
use rand::{thread_rng, Rng};

fn main() {
    let mut env = CartPoleEnv::default();
    // TODO: not sure why total_reward changes with each run as if seeding does not work.
    // this needs investigation
    env.seed(0);

    let mut _state: Vec<f64> = env.reset();

    let mut rng = thread_rng();
    let mut end: bool = false;
    let mut total_reward: f64 = 0.0;
    while !end {
        // randomly choose an action to take
        let action = ActionType::Discrete(rng.gen_range(0, 2));
        let (_state, reward, done, _info) = env.step(action);
        end = done;
        total_reward += reward;
    }
    println!("total_reward: {}", total_reward);
}