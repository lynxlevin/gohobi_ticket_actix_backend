pub const ARGON2_START_WORD: &str = "$argon2id$";
pub const DJANGO_START_WORD: &str = "pbkdf2_sha256$";
pub const USER_ID_KEY: &str = "gt_user_id";
pub const USER_EMAIL_KEY: &str = "gt_user_email";
pub const NOT_FOUND_MESSAGE: &str = "A user with these details does not exist. If you registered with these details, ensure you activate your account by clicking on the link sent to your e-mail address.";
pub const TOO_MANY_LOGIN_ATTEMPTS_MESSAGE: &str =
    "Your account is temporarily locked. Please wait for 1 hour.";
