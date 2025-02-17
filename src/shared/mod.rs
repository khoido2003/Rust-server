pub mod adjectives;
pub mod animals;

use adjectives::ADJECTIVES;
use animals::ANIMALS;

pub fn random_name() -> String {
    let adjective = fastrand::choice(ADJECTIVES).unwrap();
    let animal = fastrand::choice(ANIMALS).unwrap();

    format!("{adjective}{animal}")
}

macro_rules! b {
    ($result:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(err) => return Err(err.into()),
        }
    };
}
pub(crate) use b;
