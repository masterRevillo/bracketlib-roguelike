use crate::configs;

pub fn get_config(key: &str) -> &str {
    *&configs.get(key).unwrap().as_str()
}
