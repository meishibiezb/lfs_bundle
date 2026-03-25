#[derive(Clone, Debug, Default)]
pub struct AppTheme {
    pub accent: [u8; 3],
}

impl AppTheme {
    pub fn default_accent() -> [u8; 3] {
        [0x4f, 0x8f, 0xff]
    }
}
