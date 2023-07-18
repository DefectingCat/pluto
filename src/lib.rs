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

#[derive(Default)]
pub struct Pluto {
    pub method: PingMethod,
}

impl Pluto {}
