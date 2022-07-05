use crate::utils::Float;
use crate::utils::Float;
use crate::{scale, spaces, ActionType, GifRender, GymEnv};
use ordered_float::OrderedFloat;
use plotters::prelude::*;
use rand::distributions::Uniform;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

/**
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
    In case of discrete action:
    Num    Action
    0      Accelerate to the left
    1      Don't accelerate
    2      Accelerate to the right

    In case of continuous action, apply a force in range [-1.0, 1.0],
    1.0 being maximum acceleration to the right,
    -1.0 being maximum acceleration to the left

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
**/

#[derive(Debug)]
pub struct MountainCarEnv<T> {
    pub min_position: Float<T>,
    pub max_position: Float<T>,
    pub max_speed: Float<T>,
    pub goal_position: Float<T>,
    pub goal_velocity: Float<T>,

    pub force: Float<T>,
    pub gravity: Float<T>,

    pub low: Observation<T>,
    pub high: Observation<T>,

    // TODO: Add properties related to rendering such screen_width, screen_height, etc..
    // REFER TO: https://github.com/openai/gym/blob/master/gym/envs/classic_control/mountain_car.py
    pub state: Observation<T>,
    pub action_space: spaces::Discrete,
    pub observation_space: spaces::Box<Observation<T>>,

    rng: Pcg64,
}

// Utility structure intended to reduce confusion around meaning of properties.
pub struct Observation<T>(Float<T>, Float<T>);

impl<T> Observation<T> {
    pub fn get_position(&self) -> Float<T> {
        self.0
    }

    pub fn get_velocity(&self) -> Float<T> {
        self.1
    }
}

impl<T> MountainCarEnv<T> {
    fn new(render_mode: Option<&str>, goal_velocity: Option<Float<T>>) {
        let rng = Pcg64::from_entropy();
        let min_position = -1.2;
        let max_position = 0.6;
        let max_speed = 0.07;
        let goal_position = 0.5;
        let goal_velocity = 0.0;
        let force = 0.001;
        let gravity = 0.0025;
        let state = [0.0; 2];
        let action_space = spaces::Discrete(3);
    }
}

impl GymEnv for MountainCarEnv {
    fn step(&mut self, action: ActionType) -> (Vec<f64>, f64, bool, Option<String>) {
        let action: f64 = match action {
            ActionType::Discrete(a) => a as f64 - 1.0,
            ActionType::Continuous(a) => {
                assert_eq!(a.len(), 1, "action vector has to have length of 1");
                // continuous action in range [-1.0, 1.0]
                if a[0] < -1.0 {
                    -1.0
                } else if a[0] > 1.0 {
                    1.0
                } else {
                    a[0]
                }
            }
        };

        let mut position = self.state[0];
        let mut velocity = self.state[1];

        velocity += action * self.force + (3.0 * position).cos() * (-self.gravity);
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
        self.score += reward;

        self.state = [position, velocity];
        self.episode_length += 1;

        (self.state.to_vec(), reward, done, None)
    }

    fn reset(&mut self) -> Vec<f64> {
        let d = Uniform::new(-0.6, -0.4);
        self.state = [self.rng.sample(d), 0.0];
        self.state.to_vec()
    }

    /// render the environment using 30 frames per second
    fn render(&self, render: &mut GifRender) {
        render.drawing_area.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&render.drawing_area)
            .caption(format!("Mountain Car Environment"), ("sans-serif", 20))
            .build_cartesian_2d(-1.2..0.6, 0.0..1.0)
            .unwrap();

        // draw track
        chart
            .draw_series(LineSeries::new(
                (0..render.width)
                    .map(|x| {
                        scale(
                            0.0,
                            render.width as f64,
                            self.min_position,
                            self.max_position,
                            x as f64,
                        )
                    })
                    .map(|x| (x, (3.0 * x).sin() * 0.45 + 0.55)),
                &RED,
            ))
            .unwrap();

        // draw cart
        let cart_x: f64 = self.state[0];
        let track_y: f64 = (3.0 * cart_x).sin() * 0.45 + 0.55;
        let cart_width = 0.066;
        let cart_height = 0.05;
        chart
            .draw_series(vec![(0.0, 0.0)].iter().map(|_| {
                Rectangle::new(
                    [
                        ((cart_x - cart_width), track_y),
                        ((cart_x + cart_width), (track_y + cart_height)),
                    ],
                    HSLColor(0.8, 0.7, 0.1).filled(),
                )
            }))
            .unwrap();

        // TODO: draw finish line

        // draw score
        let style = TextStyle::from(("sans-serif", 20).into_font()).color(&RED);
        render
            .drawing_area
            .draw_text(
                &format!("Score: {}", self.score),
                &style,
                (
                    scale(0.0, 1.0, 0.0, render.width as f64, 0.1) as i32,
                    scale(0.0, 1.0, 0.0, render.height as f64, 0.9) as i32,
                ),
            )
            .unwrap();

        render.drawing_area.present().unwrap()
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
