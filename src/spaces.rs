pub struct Discrete(pub u8);

impl Discrete {
    pub fn contains(&self, value: u8) -> bool {
        value < self.0
    }
}

// TODO: Add bounds (T needs to be float convertable or a Vector of any value);
// LINK: https://github.com/openai/gym/blob/2ede09074fe72e9e0dc6790c327d3eb54335ecd0/gym/spaces/box.py#L34
pub struct Box<T>(pub T, pub T);
