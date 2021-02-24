use crate::{ActionType, GymEnv, Viewer};
use piston_window::*;
use rand::distributions::Uniform;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use std::thread;
use std::time::Duration;

/**
Description:
    The inverted pendulum swingup problem is a classic problem in the control literature.
    In this version of the problem, the pendulum starts in a random position,
    and the goal is to swing it up so it stays upright.

Source:
    OpenAI's Gym Python implementation

Observation
    Num     Observation         Min     Max
    0       Cosine of theta     -1.0    1.0
    1       Sine of theta       -1.0    1.0
    2       pendulum velocity   -Inf    Inf

Actions:

**/
pub struct PendulumEnv {
    rng: Pcg64,
    state: [f64; 2],
    max_speed: f64,
    max_torque: f64,
    dt: f64,
    g: f64,
    m: f64,
    l: f64,
}

impl Default for PendulumEnv {
    fn default() -> Self {
        Self {
            rng: Pcg64::from_entropy(),
            state: [0.0; 2],
            max_speed: 8.0,
            max_torque: 2.0,
            dt: 0.05,
            g: 10.0,
            m: 1.0,
            l: 1.0,
        }
    }
}

impl PendulumEnv {
    fn get_obs(&self) -> Vec<f64> {
        vec![self.state[0].cos(), self.state[0].sin(), self.state[1]]
    }

    fn angle_normalize(theta: f64) -> f64 {
        ((theta + std::f64::consts::PI) % (2.0 * std::f64::consts::PI)) - std::f64::consts::PI
    }
}

impl GymEnv for PendulumEnv {
    fn step(&mut self, action: ActionType) -> (Vec<f64>, f64, bool, Option<String>) {
        let mut u = match action {
            ActionType::Discrete(_) => panic!("discrete action is not supported for the pendulum environment! Please use continuous actions"),
            ActionType::Continuous(a) => a[0],
        };

        let theta: f64 = self.state[0];
        let theta_dot: f64 = self.state[1];

        // clip action to min and max torque
        if u < -self.max_torque {
            u = -self.max_torque;
        } else if u > self.max_torque {
            u = self.max_torque;
        }

        let costs: f64 =
            Self::angle_normalize(theta).powi(2) + 0.1 * theta_dot.powi(2) + 0.001 * u.powi(2);

        let mut new_theta_dot: f64 = theta_dot
            + (-3.0 * self.g / (2.0 * self.l) * (theta + std::f64::consts::PI).sin()
                + 2.0 / (self.m * self.l.powi(2)) * u)
                * self.dt;
        let new_theta = theta + new_theta_dot * self.dt;
        // clip new_theta_dot to max speed
        if new_theta_dot > self.max_speed {
            new_theta_dot = self.max_speed;
        } else if new_theta_dot < -self.max_speed {
            new_theta_dot = -self.max_speed;
        }

        self.state = [new_theta, new_theta_dot];

        (self.get_obs(), -costs, false, None)
    }

    fn reset(&mut self) -> Vec<f64> {
        let d_0 = Uniform::new(-std::f64::consts::PI, std::f64::consts::PI);
        let d_1 = Uniform::new(-1.0, 1.0);

        self.state = [self.rng.sample(d_0), self.rng.sample(d_1)];

        self.get_obs()
    }

    fn render(&self, viewer: &mut Viewer) {
        let width: f64 = viewer.window_width as f64;
        let height: f64 = viewer.window_height as f64;
        if let Some(e) = viewer.window.next() {
            viewer.window.draw_2d(&e, |c, g, _d| {
                clear([0.5, 1.0, 0.5, 1.0], g);

                let center_x: f64 = width / 2.0;
                let center_y: f64 = height / 2.0;
                let pole_len: f64 = (width / 4.0) * self.l;
                let top_x: f64 = center_x + (-pole_len * -self.state[0].sin());
                let top_y: f64 = center_y + (pole_len * -self.state[0].cos());

                // Draw pendulum
                line_from_to(
                    [0.1, 0.1, 0.1, 1.0],
                    5.0,
                    [center_x, center_y],
                    [top_x, top_y],
                    c.transform,
                    g,
                );
            });
            //
            thread::sleep(Duration::from_millis((1000.0 * self.dt) as u64));
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
    fn pendulum_render() {
        let mut env = PendulumEnv::default();

        let mut viewer = Viewer::default();

        let mut rng = thread_rng();
        let d = Uniform::new(-1.0, 1.0);
        for i in 0..200 {
            let action = ActionType::Continuous(vec![rng.sample(d)]);
            env.step(action);

            env.render(&mut viewer);
        }
    }
}
