use gym_rs::{ActionType, GifRender, GymEnv, PendulumEnv};

fn main() {
    let mut env = PendulumEnv::default();
    env.seed(0);

    let mut render = GifRender::new(540, 540, "img/pendulum_render.gif", 50).unwrap();

    let mut state: Vec<f64> = env.reset();

    let mut end: bool = false;
    let mut steps: usize = 0;
    while !end {
        if steps > 100 {
            break;
        }
        let action: ActionType = if state[0] > 0.0 {
            ActionType::Continuous(vec![-1.0])
        } else {
            ActionType::Continuous(vec![1.0])
        };
        let (s, _reward, done, _info) = env.step(action);
        end = done;
        state = s;

        env.render(&mut render);

        steps += 1;
    }
}
