#[inline]
pub fn major() -> u32 {
    env!("CARGO_PKG_VERSION_MAJOR").parse::<u32>().unwrap_or(0)
}

#[inline]
pub fn minor() -> u32 {
    env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap_or(0)
}
