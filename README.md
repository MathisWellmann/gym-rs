# OpenAI Gym written in pure Rust for blazingly fast performance :rocket:

:warning: :construction: Work in Progress :construction:

This library aims be be as close to the original OpenAI Gym library which is written in Python
and translate it into Rust for blazingly fast performance.
This will make the use of Python unnecessary which is awesome.

If you don't mind Python, check out a gym [wrapper](https://github.com/MrRobb/gym-rs).

## Installation
Put this in your Cargo.toml
```toml
gym_rs = { git = "https://www.github.com/MathisWellmann/gym-rs" }
```
If you want to render the environment like in the example below,
copy the folder "font" into your crates root directory just like in this repository, 
so that the window rendering can find the font. Otherwise rendering will panic as it cannot find the anon.ttf file.
TODO: would be nice to not rely on a font file but rather integrate it into Viewer.

## Example
Here is how you can use the cart_pole environment with a trained neural network agent from a file 
using the common genetic encoding ([cge](https://www.github.com/MathisWellmann/cge))  and rendering enabled:
```rust
/*
Cart Pole Environment solved using Neat
with a network in the form of a common genetic encoding (cge crate)
*/
extern crate cge;

use gym_rs::{CartPoleEnv, GymEnv, ActionType, Viewer};

fn main() {
    // load the network from file
    let mut net = cge::Network::load_from_file("./examples/gym_cart_pole_champion.cge").unwrap();

    let mut env = CartPoleEnv::default();

    let mut viewer = Viewer::new(1080, 1080);

    let mut state: Vec<f64> = env.reset();

    let mut end: bool = false;
    let mut total_reward: f64 = 0.0;
    while !end {
        if total_reward > 200.0 {
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
```
Run it with:
```
cargo run --release --example cart_pole
```

See [examples](https://github.com/MathisWellmann/gym-rs/tree/master/examples) folder for all the examples


## Contributions are welcome!
If you would like to add an environment or a feature, please fork this repository and create a pull request 
with your changes. Adding new environments should be as easy as translating from Python to Rust. See 
[OpenAI/gym](https://github.com/openai/gym)
for environments that are not yet implemented here! There is alot of easy work to be done here.
Any Help is highly appreciated and benefits the Rust and ML/AI community greatly!

## TODOs:
- implement more environments
- extensive documentation
- compare performance to gym-rs python wrapper
- make generic implementation and compare f32 vs f64 performance.
- publish on crates.io
- make piston_window dependency optional by introducing a render feature if possible
- remove the need for a font file

## License
gym-rs is licensed under MIT License just like OpenAI's Gym.

See [LICENSE.md](https://github.com/MathisWellmann/gym-rs/blob/master/LICENSE.md) for further details.
