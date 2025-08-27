use std::env;

pub struct SftpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub private_key: String,
    pub passphrase: Option<String>, // Optional passphrase -> Not used here
}

impl SftpConfig {
    pub fn from_env() -> Self {
        let private_key = env::var("VM_SFTP_PRIVATE_KEY")
            .expect("VM_SFTP_PRIVATE_KEY not set")
            .replace("\\n", "\n"); // restore newlines

        let passphrase = env::var("VM_SFTP_KEY_PASSPHRASE").ok();

        Self {
            host: env::var("VM_SFTP_HOST").expect("VM_SFTP_HOST not set"),
            port: env::var("VM_SFTP_PORT")
                .unwrap_or_else(|_| "22".to_string())
                .parse()
                .expect("Invalid VM_SFTP_PORT"),
            username: env::var("VM_SFTP_USER").expect("VM_SFTP_USER not set"),
            private_key,
            passphrase,
        }
    }
}
