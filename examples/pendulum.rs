use cosyne::{Activation, Config, Cosyne, Environment, ANN, PermutationProbF};
use gym_rs::{ActionType, GifRender, GymEnv, PendulumEnv};
use std::time::Instant;

fn main() {
    pretty_env_logger::init();

    let config = Config{
        pop_size: 100,
        top_ratio_to_recombine: 0.25,
        mutation_prob: 0.2,
        mutation_strength: 1.0,
        perturb_prob: 0.5,
        permutation_prob_f: PermutationProbF::Relative,
    };
    let env = Box::new(PendulumEvaluator {});
    let mut nn = ANN::new(3, 1, Activation::Relu);
    nn.add_layer(5, Activation::Relu);
    nn.add_layer(3, Activation::Relu);
    let mut cosyne = Cosyne::new(env, nn, config);
    let t0 = Instant::now();
    for _ in 0..1000 {
        cosyne.evolve();
    }
    let champion = cosyne.champion();
    println!("champion: {:?}", champion);
    println!("training time: {}s", t0.elapsed().as_secs());

    render_champion(&mut champion.0.clone());
}

struct PendulumEvaluator {}

impl Environment for PendulumEvaluator {
    fn evaluate(&self, nn: &mut ANN) -> f64 {
        let mut env = PendulumEnv::default();
        env.seed(0);

        let mut state: Vec<f64> = env.reset();

        let mut end: bool = false;
        let mut total_reward: f64 = 0.0;
        let mut steps: usize = 0;
        while !end {
            if steps > 300 {
                break;
            }
            let output = nn.forward(state);
            let action = ActionType::Continuous(vec![output[0] * 4.0 - 2.0]);
            let (s, reward, done, _info) = env.step(action);
            end = done;
            state = s;
            total_reward += reward;
            steps += 1;
        }
        total_reward
    }
}

fn render_champion(champion: &mut ANN) {
    println!("rendering champion...");

    let mut env = PendulumEnv::default();
    env.seed(0);

    let mut render = GifRender::new(540, 540, "img/pendulum_champion.gif", 50).unwrap();

    let mut state: Vec<f64> = env.reset();

    let mut end: bool = false;
    let mut steps: usize = 0;
    while !end {
        if steps > 300 {
            break;
        }
        let output = champion.forward(state);
        let action = ActionType::Continuous(vec![output[0] * 4.0 - 2.0]);
        let (s, _reward, done, _info) = env.step(action);
        end = done;
        state = s;
        steps += 1;

        env.render(&mut render);
    }
}
