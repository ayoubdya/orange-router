use sha2::{Digest, Sha256};

pub fn sha256(s: String) -> String {
  Sha256::digest(s.as_bytes())
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect()
}

pub fn bps_to_mbps(s: &str) -> String {
  let mbps: f64 = s.parse().unwrap();
  format!("{:.2}", mbps / 8388608.0) // 8388608 = 8 * 1024 * 1024
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_sha256() {
    assert_eq!(
      sha256("hello".into()),
      "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
  }

  #[test]
  fn test_bps_to_mbps() {
    assert_eq!(bps_to_mbps("16777216.0"), "2.00");
    assert_eq!(bps_to_mbps("8388608.0"), "1.00");
  }
}
