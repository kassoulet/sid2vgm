pub mod cli;
pub mod renderer;

#[cfg(test)]
mod security_test;

pub fn is_gzip(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}
