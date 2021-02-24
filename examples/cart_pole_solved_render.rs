/*
Cart Pole Environment solved using Neat
with a network in the form of a common genetic encoding (cge crate)
*/
extern crate cge;

use gym_rs::{ActionType, CartPoleEnv, GymEnv, Viewer};

fn main() {
    // load the network from file
    let mut net = cge::Network::load_from_file("./examples/gym_cart_pole_champion.cge").unwrap();

    let mut env = CartPoleEnv::default();

    let mut viewer = Viewer::new(1080, 1080);

    let mut state: Vec<f64> = env.reset();

    let mut end: bool = false;
    let mut total_reward: f64 = 0.0;
    while !end {
        if total_reward > 500.0 {
            println!("SOLVED!");
            break;
        }
        let output = net.evaluate(&state);
        let action: ActionType = if output[0] < 0.0 {
            ActionType::Discrete(0)
        } else {
            ActionType::Discrete(1)
        };
        let (s, reward, done, _info) = env.step(action);
        end = done;
        state = s;
        total_reward += reward;

        env.render(&mut viewer);
    }
    println!("total_reward: {}", total_reward);
}
