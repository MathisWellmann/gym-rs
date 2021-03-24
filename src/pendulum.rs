use crate::{scale, ActionType, GifRender, GymEnv};
use plotters::prelude::*;
use rand::distributions::Uniform;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

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
    Continuous action in range [-max_torque, max_torque] = [-2.0, 2.0]
**/
#[derive(Debug)]
pub struct PendulumEnv {
    rng: Pcg64,
    state: [f64; 2],
    max_speed: f64,
    max_torque: f64,
    dt: f64,
    g: f64,
    m: f64,
    l: f64,
    step_count: usize,
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
            step_count: 0,
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
                + 3.0 / (self.m * self.l.powi(2)) * u)
                * self.dt;
        let new_theta = theta + new_theta_dot * self.dt;
        // clip new_theta_dot to max speed
        if new_theta_dot > self.max_speed {
            new_theta_dot = self.max_speed;
        } else if new_theta_dot < -self.max_speed {
            new_theta_dot = -self.max_speed;
        }

        self.state = [new_theta, new_theta_dot];
        self.step_count += 1;

        (self.get_obs(), -costs, false, None)
    }

    fn reset(&mut self) -> Vec<f64> {
        let d_0 = Uniform::new(-std::f64::consts::PI, std::f64::consts::PI);
        let d_1 = Uniform::new(-1.0, 1.0);

        self.state = [self.rng.sample(d_0), self.rng.sample(d_1)];

        self.get_obs()
    }

    fn render(&self, render: &mut GifRender) {
        render.drawing_area.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&render.drawing_area)
            .caption(format!("Pendulum Environment"), ("sans-serif", 20))
            .build_cartesian_2d(0.0..1.0, 0.0..1.0)
            .unwrap();

        let pendulum_tip_x: f64 = 0.5 + self.state[0].sin() * 0.5;
        let pendulum_tip_y: f64 = 0.5 + self.state[0].cos() * 0.5;
        chart
            .draw_series(LineSeries::new(
                vec![(0.5, 0.5), (pendulum_tip_x, pendulum_tip_y)],
                &RED,
            ))
            .unwrap();

        // draw score
        let style = TextStyle::from(("sans-serif", 20).into_font()).color(&RED);
        render
            .drawing_area
            .draw_text(
                &format!("step: {:.2}", self.step_count),
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
