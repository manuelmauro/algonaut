use crate::constant::APPID_PREFIX;
use sha2::{Digest, Sha512};

pub fn get_application_address(app_id: u64) -> Vec<u8> {
    let app_id: &[u8; 8] = &app_id.to_be_bytes();
    let to_sign: Vec<u8> = APPID_PREFIX
        .iter()
        .copied()
        .chain(app_id.iter().copied())
        .collect();

    let mut hasher = Sha512::new();
    hasher.update(to_sign);

    hasher.finalize()[..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::get_application_address;

    #[test]
    fn get_application_address_snapshots() {
        let a = get_application_address(13);
        insta::assert_yaml_snapshot!(a);
    }
}
