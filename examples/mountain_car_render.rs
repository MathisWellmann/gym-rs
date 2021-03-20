use gym_rs::{ActionType, GifRender, GymEnv, MountainCarEnv};

fn main() {
    let mut env = MountainCarEnv::default();

    let mut render = GifRender::new(540, 540, "img/mountain_car_render.gif", 50).unwrap();

    let mut state = env.reset();

    let mut end: bool = false;
    let mut steps: usize = 0;
    while !end {
        if steps > 100 {
            break;
        }
        let action = if state[0] < -0.5 {
            ActionType::Discrete(1)
        } else {
            ActionType::Discrete(2)
        };
        let (s, _reward, done, _info) = env.step(action);
        end = done;
        state = s;

        env.render(&mut render);
        steps += 1;
    }
}
