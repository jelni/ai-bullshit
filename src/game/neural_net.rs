use serde::{Deserialize, Serialize};
use rand::{Rng, SeedableRng};

#[derive(Serialize, Deserialize, Clone)]
pub struct NeuralNet {
    pub input_weights: Vec<Vec<f32>>, // input layer to hidden layer
    pub hidden_weights: Vec<Vec<f32>>, // hidden layer to output layer
    pub input_biases: Vec<f32>,
    pub hidden_biases: Vec<f32>,
}

impl Default for NeuralNet {
    fn default() -> Self {
        Self::new(24, 16, 4) // 24 inputs, 16 hidden, 4 outputs (Up, Down, Left, Right)
    }
}

impl NeuralNet {
    pub fn new(inputs: usize, hidden: usize, outputs: usize) -> Self {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let mut input_weights = vec![vec![0.0; inputs]; hidden];
        let mut hidden_weights = vec![vec![0.0; hidden]; outputs];
        let mut input_biases = vec![0.0; hidden];
        let mut hidden_biases = vec![0.0; outputs];

        for i in 0..hidden {
            input_biases[i] = rng.gen_range(-1.0..1.0);
            for j in 0..inputs {
                input_weights[i][j] = rng.gen_range(-1.0..1.0);
            }
        }

        for i in 0..outputs {
            hidden_biases[i] = rng.gen_range(-1.0..1.0);
            for j in 0..hidden {
                hidden_weights[i][j] = rng.gen_range(-1.0..1.0);
            }
        }

        Self {
            input_weights,
            hidden_weights,
            input_biases,
            hidden_biases,
        }
    }

    fn relu(x: f32) -> f32 {
        if x > 0.0 { x } else { 0.0 }
    }

    pub fn predict(&self, inputs: &[f32]) -> usize {
        let hidden_nodes = self.input_biases.len();
        let output_nodes = self.hidden_biases.len();
        let input_nodes = inputs.len();

        let mut hidden = vec![0.0; hidden_nodes];
        for i in 0..hidden_nodes {
            let mut sum = self.input_biases[i];
            for j in 0..input_nodes {
                sum += inputs[j] * self.input_weights[i][j];
            }
            hidden[i] = Self::relu(sum);
        }

        let mut outputs = vec![0.0; output_nodes];
        for i in 0..output_nodes {
            let mut sum = self.hidden_biases[i];
            for j in 0..hidden_nodes {
                sum += hidden[j] * self.hidden_weights[i][j];
            }
            outputs[i] = sum; // No activation for output layer, just raw scores
        }

        // Find argmax
        let mut best_idx = 0;
        let mut best_val = f32::NEG_INFINITY;
        for (i, &val) in outputs.iter().enumerate() {
            if val > best_val {
                best_val = val;
                best_idx = i;
            }
        }
        best_idx
    }

    pub fn mutate(&mut self, mutation_rate: f32) {
        let mut rng = rand::rngs::StdRng::from_entropy();
        for row in &mut self.input_weights {
            for val in row {
                if rng.gen_bool(f64::from(mutation_rate)) {
                    *val += rng.gen_range(-0.5..0.5);
                }
            }
        }
        for row in &mut self.hidden_weights {
            for val in row {
                if rng.gen_bool(f64::from(mutation_rate)) {
                    *val += rng.gen_range(-0.5..0.5);
                }
            }
        }
        for val in &mut self.input_biases {
            if rng.gen_bool(f64::from(mutation_rate)) {
                *val += rng.gen_range(-0.5..0.5);
            }
        }
        for val in &mut self.hidden_biases {
            if rng.gen_bool(f64::from(mutation_rate)) {
                *val += rng.gen_range(-0.5..0.5);
            }
        }
    }

    pub fn crossover(parent1: &NeuralNet, parent2: &NeuralNet) -> NeuralNet {
        let mut rng = rand::rngs::StdRng::from_entropy();
        let mut child = parent1.clone();

        for i in 0..child.input_weights.len() {
            for j in 0..child.input_weights[i].len() {
                if rng.gen_bool(0.5) {
                    child.input_weights[i][j] = parent2.input_weights[i][j];
                }
            }
        }
        for i in 0..child.hidden_weights.len() {
            for j in 0..child.hidden_weights[i].len() {
                if rng.gen_bool(0.5) {
                    child.hidden_weights[i][j] = parent2.hidden_weights[i][j];
                }
            }
        }
        for i in 0..child.input_biases.len() {
            if rng.gen_bool(0.5) {
                child.input_biases[i] = parent2.input_biases[i];
            }
        }
        for i in 0..child.hidden_biases.len() {
            if rng.gen_bool(0.5) {
                child.hidden_biases[i] = parent2.hidden_biases[i];
            }
        }
        child
    }
}
