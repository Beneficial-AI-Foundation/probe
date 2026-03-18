pub struct Config {
    pub name: String,
    pub value: u64,
}

pub type Alias = Vec<Config>;

pub enum Status {
    Active,
    Inactive,
    Pending(String),
}

pub const MAX_RETRIES: u32 = 3;
