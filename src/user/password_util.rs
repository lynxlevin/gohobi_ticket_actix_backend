use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::Engine, prelude::BASE64_STANDARD};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

use crate::constants::{ARGON2_START_WORD, DJANGO_START_WORD};

#[derive(PartialEq, Debug)]
pub enum PasswordType {
    Argon2,
    Django,
    Unknown,
}

pub fn verify(password: &str, hashed_password: &str) -> Result<PasswordType, PasswordType> {
    if hashed_password.starts_with(ARGON2_START_WORD) {
        match verify_argon2_password(password, hashed_password) {
            true => Ok(PasswordType::Argon2),
            false => Err(PasswordType::Argon2),
        }
    } else if hashed_password.starts_with(DJANGO_START_WORD) {
        match verify_django_password(password, hashed_password) {
            true => Ok(PasswordType::Django),
            false => Err(PasswordType::Django),
        }
    } else {
        Err(PasswordType::Unknown)
    }
}

fn verify_argon2_password(password: &str, argon2_password: &str) -> bool {
    let parse_result = PasswordHash::new(argon2_password);
    match parse_result {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}

fn verify_django_password(password: &str, django_password: &str) -> bool {
    let iterations = django_password[14..20].parse::<u32>().unwrap();
    let salt = &django_password[21..43];
    let mut keys = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt.as_bytes(), iterations, &mut keys);
    let b64 = BASE64_STANDARD.encode(keys);

    let new_encoded_password = format!("pbkdf2_sha256${}${}${}", iterations, salt, b64,);
    django_password == new_encoded_password
}

pub fn encode_argon2(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let result = argon2.hash_password(password.as_bytes(), &salt);
    match result {
        Ok(password_hash) => Ok(password_hash.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_with_argon2_password() {
        let password = "password";
        let argon2_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";

        let res = verify(password, argon2_password);

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PasswordType::Argon2);
    }

    #[test]
    fn test_verify_with_incorrect_argon2_password() {
        let incorrect_password = "passworda";
        let argon2_password = "$argon2id$v=19$m=19456,t=2,p=1$r07vWFCaKrbNPrSgUrG/+Q$/2lBaeRWeox6ROMu6qAwOYmttdGXA3o4Uw2YHC/fvfY";

        let res = verify(incorrect_password, argon2_password);

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), PasswordType::Argon2);
    }

    #[test]
    #[ignore]
    fn test_verify_with_django_password() {
        // Ignore this test because it's very slow
        let password = "password";
        let django_password = "pbkdf2_sha256$260000$N4b3mSYc5bXPsCkD7G3eKt$4nfua4vv7GLRqeRHxCcDmjtMxB6LYZNhMf6Lqh48RDE=";

        let res = verify(password, django_password);

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PasswordType::Django);
    }

    #[test]
    #[ignore]
    fn test_verify_with_incorrect_django_password() {
        // Ignore this test because it's very slow
        let incorrect_password = "passworda";
        let django_password = "pbkdf2_sha256$260000$N4b3mSYc5bXPsCkD7G3eKt$4nfua4vv7GLRqeRHxCcDmjtMxB6LYZNhMf6Lqh48RDE=";

        let res = verify(incorrect_password, django_password);

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), PasswordType::Django);
    }

    #[test]
    fn test_verify_with_unknown_password_hash() {
        let password = "password";
        let unknown_password = "apbkdf2_sha256$260000$N4b3mSYc5bXPsCkD7G3eKt$4nfua4vv7GLRqeRHxCcDmjtMxB6LYZNhMf6Lqh48RDE=";

        let res = verify(password, unknown_password);

        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), PasswordType::Unknown);
    }

    #[test]
    fn test_encode_argon2() {
        let password = "password";

        let result = encode_argon2(password);

        assert!(result.is_ok());
        let hashed_password = result.unwrap();
        assert!(hashed_password.starts_with(ARGON2_START_WORD));
        assert!(Argon2::default()
            .verify_password(
                password.as_bytes(),
                &PasswordHash::new(&hashed_password).unwrap()
            )
            .is_ok());
    }
}
