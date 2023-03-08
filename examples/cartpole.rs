use gym_rs::{
    core::Env, envs::classical_control::cartpole::CartPoleEnv, utils::renderer::RenderMode,
};
use log::debug;
use ordered_float::OrderedFloat;
use rand::{thread_rng, Rng};

fn main() {
    pretty_env_logger::try_init().unwrap_or(());

    let mut env = CartPoleEnv::new();
    env.reset(None, false, None);

    let mut rewards = vec![];

    for _ in 0..15 {
        let mut current_reward = OrderedFloat(0.);

        for _ in 0..500 {
            let action = (&mut thread_rng()).gen_range(0..=1);
            let state_reward = env.step(action);
            current_reward += state_reward.reward;

            if state_reward.done {
                break;
            }
        }

        env.reset(None, false, None);
        rewards.push(current_reward);
    }

    debug!("{:?}", rewards)
}
