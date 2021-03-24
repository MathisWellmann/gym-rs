use cosyne::{Activation, Config, Cosyne, Environment, ANN};
use gym_rs::{ActionType, CartPoleEnv, GifRender, GymEnv};
use std::time::Instant;

fn main() {
    pretty_env_logger::init();

    let config = Config::new(100);
    let env = Box::new(CartPoleEvaluator {});
    let nn = ANN::new(4, 1, Activation::Relu);
    let mut cosyne = Cosyne::new(env, nn, config);
    let t0 = Instant::now();
    for _ in 0..100 {
        cosyne.evolve();
    }
    let champion = cosyne.champion();
    println!("champion: {:?}", champion);
    println!("training time: {}ms", t0.elapsed().as_secs());
    assert!(champion.1 >= 400.0);

    render_champion(&mut champion.0.clone());
}

struct CartPoleEvaluator {}

impl Environment for CartPoleEvaluator {
    fn evaluate(&self, nn: &mut ANN) -> f64 {
        let mut env = CartPoleEnv::default();

        let mut state: Vec<f64> = env.reset();

        let mut end: bool = false;
        let mut total_reward: f64 = 0.0;
        while !end {
            if total_reward >= 400.0 {
                break;
            }
            let output = nn.forward(state);
            let action: ActionType = if output[0] < 0.5 {
                ActionType::Discrete(0)
            } else {
                ActionType::Discrete(1)
            };
            let (s, reward, done, _info) = env.step(action);
            end = done;
            state = s;
            total_reward += reward;
        }
        total_reward
    }
}

fn render_champion(champion: &mut ANN) {
    println!("rendering champion...");

    let mut env = CartPoleEnv::default();

    let mut render = GifRender::new(540, 540, "img/cart_pole_champion.gif", 20).unwrap();

    let mut state: Vec<f64> = env.reset();

    let mut end: bool = false;
    let mut steps: usize = 0;
    while !end {
        if steps > 300 {
            break;
        }
        let output = champion.forward(state);
        let action: ActionType = if output[0] < 0.5 {
            ActionType::Discrete(0)
        } else {
            ActionType::Discrete(1)
        };
        let (s, _reward, done, _info) = env.step(action);
        end = done;
        state = s;
        steps += 1;

        env.render(&mut render);
    }
}
