mod crypto;

pub use crypto::detection;
pub use crypto::errors;
pub use crypto::key_dec::*;
pub use crypto::qmc2::decrypt_factory;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
