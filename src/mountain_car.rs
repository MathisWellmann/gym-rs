use crate::{GymEnv, ActionType, Viewer};
use crate::utils::scale;
use rand_pcg::Pcg64;
use rand::{SeedableRng, Rng};
use rand::distributions::Uniform;
use piston_window::*;
use std::time::Duration;
use std::thread;

/*
Description:
    The agent (a car) is started at the bottom of a valley. For any given
    state, the agent may choose to accelerate to the left, right or cease
    any acceleration.

Source:
    The environment appeared first in Andrew Moore's PhD Thesis (1990).
    the source code in python: https://www.github.com/openai/gym

Observation:
    Num     Observation         Min     Max
    0       Car Position        -1.2    0.6
    1       Car Velocity        -0.07   0.07

Actions:
    Num    Action
    0      Accelerate to the left
    1      Don't accelerate
    2      Accelerate to the right

    Note: This does not affect the amount of velocity affected by the gravitational pull acting on the cart.

Reward:
    Reward of 0 is awarded if the agent reached the flag (position = 0.5)
    on top of the mountain.
    Reward of -1 is awarded if the position of the agent is less than 0.5.

Starting State:
    The position of the car is assigned a uniform random value in
    [-0.6, -0.4].
    The starting velocity of the car is always assigned to 0.

Episode Termination:
    The car position is more than 0.5
    Episode length is greater than 200
*/
pub struct MountainCarEnv {
    rng: Pcg64,
    min_position: f64,
    max_position: f64,
    max_speed: f64,
    goal_position: f64,
    goal_velocity: f64,
    force: f64,
    gravity: f64,
    state: [f64; 2],
    episode_length: usize,
}

impl MountainCarEnv {
    pub fn episode_length(&self) -> usize {
        self.episode_length
    }
}

impl Default for MountainCarEnv {
    fn default() -> Self {
        Self {
            rng: Pcg64::from_entropy(),
            min_position: -1.2,
            max_position: 0.6,
            max_speed: 0.07,
            goal_position: 0.5,
            goal_velocity: 0.0,
            force: 0.001,
            gravity: 0.0025,
            state: [0.0; 2],
            episode_length: 0,
        }
    }
}

impl GymEnv for MountainCarEnv {
    fn step(&mut self, action: ActionType) -> (Vec<f64>, f64, bool, Option<String>) {
        let action: u8 = match action {
            ActionType::Discrete(a) => a,
            ActionType::Continuous(_) => panic!("continuous action is not supported in this environment"),
        };

        let mut position = self.state[0];
        let mut velocity = self.state[1];

        velocity += (action as f64 - 1.0) * self.force + (3.0 * position).cos() * (-self.gravity);
        if velocity > self.max_speed {
            velocity = self.max_speed;
        } else if velocity < -self.max_speed {
            velocity = -self.max_speed;
        }

        position += velocity;
        if position < self.min_position {
            position = self.min_position;
        } else if position > self.max_position {
            position = self.max_position;
        }
        if position == self.min_position && velocity < 0.0 {
            velocity = 0.0;
        }

        let done: bool = position >= self.goal_position && velocity >= self.goal_velocity;
        let reward: f64 = -1.0;

        self.state = [position, velocity];
        self.episode_length += 1;

        (self.state.to_vec(), reward, done, None)
    }

    fn reset(&mut self) -> Vec<f64> {
        let d = Uniform::new(-0.6, -0.4);
        self.state = [
            self.rng.sample(d),
            0.0
        ];
        self.state.to_vec()
    }

    /// render the environment using 30 frames per second
    fn render(&self, viewer: &mut Viewer) {
        if let Some(e) = viewer.window.next() {
            let cart_width = scale(0.0, 1.0, 0.0, viewer.window_width as f64, 0.066);
            let cart_height = scale(0.0, 1.0, 0.0, viewer.window_height as f64, 0.05);
            let width = viewer.window_width as f64;
            let height = viewer.window_height as f64;

            viewer.window.draw_2d(&e, |c, g, d| {
                clear([0.5, 1.0, 0.5, 1.0], g);

                // draw track
                let xs: Vec<f64> = (0..100)
                    .map(|i| scale::<f64>(0.0, 100.0, self.min_position, self.max_position, i as f64))
                    .collect::<Vec<f64>>();
                let ys: Vec<f64> = xs.iter()
                    .map(|v| (3.0 * v).sin() * 0.45 + 0.55)
                    .collect();
                let xys: Vec<[f64; 2]> = xs.iter()
                    .zip(&ys)
                    .map(|(x, y)| {
                        let x_scaled: f64 = scale(self.min_position, self.max_position, 0.0, width, *x);
                        let y_scaled: f64 = height - scale(0.0, 1.0, 0.0, height, *y);
                        [x_scaled, y_scaled]
                    })
                    .collect();
                // Draw path one line at a time
                for (i, xy) in xys.iter().enumerate() {
                    if i == 0 { continue }
                    line_from_to([0.0, 0.0, 0.0, 1.0],
                                 1.0,
                                 *xy,
                                 xys[i - 1],
                                 c.transform,
                                 g);
                }

                // draw cart
                let cart_x: f64 = scale(-1.2, 0.6, 0.0, width, self.state[0]);
                let cart_y: f64 = height - scale(0.0, 1.0, 0.0, height, (3.0 * self.state[0]).sin() * 0.45 + 0.55);  // TODO: map to curve
                rectangle([0.1, 0.1, 0.1, 1.0],
                          [cart_x - cart_width / 2.0, cart_y - cart_height / 2.0, cart_width, cart_height],
                          c.transform,
                          g);

                // TODO: rotate cart body depending on hill angle

                // TODO: draw finish

                // TODO: draw wheels
            });
            // run at ~30 frames per second
            thread::sleep(Duration::from_millis(33));
        }
    }

    fn seed(&mut self, seed: u64) {
        self.rng = Pcg64::seed_from_u64(seed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn mountain_car() {
        let mut mc = MountainCarEnv::default();
        let _state = mc.reset();

        let mut rng = thread_rng();
        let mut end: bool = false;
        while !end {
            if mc.episode_length > 200 {
                break;
            }
            let action = match rng.gen_range(0, 3) {
                0 => ActionType::Discrete(0),
                1 => ActionType::Discrete(1),
                _ => ActionType::Discrete(2),
            };
            let (_state, _r, done, _) = mc.step(action);
            end = done;
            println!("episode_length: {}", mc.episode_length);
        }
    }
}
