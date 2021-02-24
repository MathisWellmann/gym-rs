extern crate find_folder;

use crate::{scale, ActionType, GymEnv, Viewer};
use piston_window::*;
use rand::distributions::Uniform;
use rand::prelude::*;
use rand_pcg::Pcg64;
use std::thread;
use std::time::Duration;

/*
Description:
    A pole is attached by an un-actuated joint to a cart, which moves along
    a frictionless track. The pendulum starts upright, and the goal is to
    prevent it from falling over by increasing and reducing the cart's
    velocity.

Source:
    https://github.com/openai/gym
    This environment corresponds to the version of the cart-pole problem
    described by Barto, Sutton, and Anderson

Observation:
    Type: Vec<f64>
    Index   Observation             Min                     Max
    0       Cart Position           -4.8                    4.8
    1       Cart Velocity           -Inf                    Inf
    2       Pole Angle              -0.418 rad (-24 deg)    0.418 rad (24 deg)
    3       Pole Angular Velocity   -Inf                    Inf

Actions:
    Type: Discrete(2)
        Num   Action
        0     Push cart to the left
        1     Push cart to the right

    Note: The amount the velocity that is reduced or increased is not
    fixed; it depends on the angle the pole is pointing. This is because
    the center of gravity of the pole increases the amount of energy needed
    to move the cart underneath it

Reward:
    Reward is 1 for every step taken, including the termination step

Starting State:
    All observations are assigned a uniform random value in [-0.05..0.05]

 Episode Termination:
    Pole Angle is more than 12 degrees.
    Cart Position is more than 2.4 (center of the cart reaches the edge of
    the display).
    Episode length is greater than 200.
    Solved Requirements:
    Considered solved when the average return is greater than or equal to
    195.0 over 100 consecutive trials.
*/
#[derive(Debug)]
/// The cart pole environment
pub struct CartPoleEnv {
    gravity: f64,
    mass_cart: f64,
    mass_pole: f64,
    total_mass: f64,
    length: f64, // actually half the pole's length
    pole_mass_length: f64,
    force_mag: f64,
    tau: f64, // seconds between state updates
    kinematics_integrator: KinematicsIntegrator,
    // Angle at which to fail the episode
    theta_threshold_radians: f64,
    x_threshold: f64,
    rng: Pcg64,
    state: [f64; 4],
    steps_beyond_done: Option<usize>,
    score: f64, // cumulative reward used to rendering to window
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum KinematicsIntegrator {
    Euler,
    SemiImplicitEuler,
}

impl Default for CartPoleEnv {
    fn default() -> Self {
        let mass_cart: f64 = 1.0;
        let mass_pole: f64 = 0.1;
        let length: f64 = 0.5;
        Self {
            gravity: 9.8,
            mass_cart,
            mass_pole,
            total_mass: mass_cart + mass_pole,
            length,
            pole_mass_length: mass_pole * length,
            force_mag: 10.0,
            tau: 0.02,
            kinematics_integrator: KinematicsIntegrator::Euler,
            theta_threshold_radians: 12.0 * 2.0 * std::f64::consts::PI / 360.0,
            x_threshold: 2.4,
            rng: Pcg64::from_entropy(),
            state: [0.0; 4],
            steps_beyond_done: None,
            score: 0.0,
        }
    }
}

impl GymEnv for CartPoleEnv {
    fn step(&mut self, action: ActionType) -> (Vec<f64>, f64, bool, Option<String>) {
        let action = match action {
            ActionType::Discrete(v) => v,
            ActionType::Continuous(_) => panic!("wrong action type provided"),
        };

        let mut x = self.state[0];
        let mut x_dot = self.state[1];
        let mut theta = self.state[2];
        let mut theta_dot = self.state[3];

        let force: f64 = if action == 1 {
            self.force_mag
        } else {
            -self.force_mag
        };
        let cos_theta: f64 = theta.cos();
        let sin_theta: f64 = theta.sin();

        let temp: f64 =
            (force + self.pole_mass_length * theta_dot.powi(2) * sin_theta) / self.total_mass;
        let theta_acc: f64 = (self.gravity * sin_theta - cos_theta * temp)
            / (self.length * (4.0 / 3.0 - self.mass_pole * cos_theta.powi(2) / self.total_mass));
        let x_acc: f64 = temp - self.pole_mass_length * theta_acc * cos_theta / self.total_mass;

        match self.kinematics_integrator {
            KinematicsIntegrator::Euler => {
                x += self.tau * x_dot;
                x_dot += self.tau * x_acc;
                theta += self.tau * theta_dot;
                theta_dot += self.tau * theta_acc;
            }
            KinematicsIntegrator::SemiImplicitEuler => {
                x_dot += self.tau * x_acc;
                x += self.tau * x_dot;
                theta_dot += self.tau * theta_acc;
                theta += self.tau * theta_dot;
            }
        }
        self.state = [x, x_dot, theta, theta_dot];

        let done: bool = x < -self.x_threshold
            || x > self.x_threshold
            || theta < -self.theta_threshold_radians
            || theta > self.theta_threshold_radians;

        let reward: f64 = if !done {
            1.0
        } else if self.steps_beyond_done.is_none() {
            // pole just fell
            self.steps_beyond_done = Some(0);
            1.0
        } else {
            if self.steps_beyond_done.unwrap() == 0 {
                warn!(
                    "You are calling 'step()' even though this \
                environment has already returned done = true. You should always call 'reset()' \
                once you receive 'done = true' -- any further steps are undefined behaviour"
                );
            }
            0.0
        };
        self.score += reward;

        (self.state.to_vec(), reward, done, None)
    }

    fn reset(&mut self) -> Vec<f64> {
        let d = Uniform::new(-0.05, 0.05);
        self.state = [
            self.rng.sample(d),
            self.rng.sample(d),
            self.rng.sample(d),
            self.rng.sample(d),
        ];
        self.steps_beyond_done = None;
        self.score = 0.0;

        self.state.to_vec()
    }

    fn render(&self, viewer: &mut Viewer) {
        if let Some(e) = viewer.window.next() {
            let width: f64 = viewer.window_width as f64;
            let track_y: f64 = 0.75 * viewer.window_height as f64;
            let cart_x: f64 = scale(-2.4, 2.4, 0.0, viewer.window_width as f64, self.state[0]);
            let cart_width: f64 = scale(0.0, 1.0, 0.0, viewer.window_width as f64, 0.0833);
            let cart_height: f64 = scale(0.0, 1.0, 0.0, viewer.window_height as f64, 0.075);

            // in original gym implementation pole_len gets multiplied by 2.0, but this renders it too long
            let pole_len: f64 = (viewer.window_width as f64 / self.x_threshold * 2.0) * self.length;

            let pole_top_x: f64 = cart_x + (-pole_len * -self.state[2].sin());
            let pole_top_y: f64 = track_y - (pole_len * -self.state[2].cos());

            let glyphs = &mut viewer.glyphs;
            viewer.window.draw_2d(&e, |c, g, d| {
                clear([0.5, 1.0, 0.5, 1.0], g);

                // draw track
                rectangle(
                    [0.1, 0.1, 0.1, 1.0],        // color
                    [0.0, track_y, width, 10.0], // [x, y, w, h]
                    c.transform,
                    g,
                );

                // draw cart
                rectangle(
                    [0.05, 0.05, 0.05, 1.0],
                    [
                        cart_x - cart_width / 2.0,
                        track_y - cart_height / 2.0,
                        cart_width,
                        cart_height,
                    ],
                    c.transform,
                    g,
                );

                // draw pole
                line(
                    [0.859, 0.506, 0.0, 1.0],
                    5.0,
                    [
                        cart_x,
                        track_y,
                        pole_top_x,
                        track_y - (track_y - pole_top_y).abs(),
                    ],
                    c.transform,
                    g,
                );

                // draw score
                text::Text::new_color([0.0, 1.0, 0.0, 1.0], 32)
                    .draw(
                        &format!("SCORE: {}", self.score),
                        glyphs,
                        &c.draw_state,
                        c.transform,
                        g,
                    )
                    .unwrap();

                // update glyphs before rendering
                glyphs.factory.encoder.flush(d);
            });

            thread::sleep(Duration::from_millis((self.tau * 1000.0) as u64));
        }
    }

    fn seed(&mut self, seed: u64) {
        self.rng = Pcg64::seed_from_u64(seed);
    }
}
