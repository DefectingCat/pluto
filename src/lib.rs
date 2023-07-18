#[derive(Default)]
pub enum PingMethod {
    #[default]
    Tcp,
    Http,
    Unknow,
}
impl From<&str> for PingMethod {
    fn from(value: &str) -> Self {
        use PingMethod::*;
        match value.to_lowercase().as_str() {
            "tcp" => Tcp,
            "http" => Http,
            _ => Unknow,
        }
    }
}

pub struct Pluto {
    pub method: PingMethod,
    pub port: u32,
}
impl Default for Pluto {
    fn default() -> Self {
        Self {
            method: PingMethod::default(),
            port: 80
        }
    }
    
}

impl Pluto {}
