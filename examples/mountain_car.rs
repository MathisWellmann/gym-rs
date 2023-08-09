use gym_rs::{
    core::{ActionReward, Env},
    envs::classical_control::mountain_car::MountainCarEnv,
    utils::renderer::RenderMode,
};
use rand::{thread_rng, Rng};

fn main() {
    let mut mc = MountainCarEnv::new(RenderMode::Human);
    let _state = mc.reset(None, false, None);

    let mut end: bool = false;
    let mut episode_length = 0;
    while !end {
        if episode_length > 200 {
            break;
        }
        let action = (&mut thread_rng()).gen_range(0..3);
        let ActionReward { done, .. } = mc.step(action);
        episode_length += 1;
        end = done;
        println!("episode_length: {}", episode_length);
    }

    mc.close();

    for _ in 0..200 {
        let action = (&mut thread_rng()).gen_range(0..3);
        mc.step(action);
        episode_length += 1;
        println!("episode_length: {}", episode_length);
    }
}
