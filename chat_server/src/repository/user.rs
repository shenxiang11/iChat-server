use std::mem;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use jwt_simple::reexports::rand;
use lettre::message::MessageBuilder;
use lettre::{SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;
use r2d2_redis::redis::Commands;
use sqlx::PgPool;
use tracing::log::debug;
use crate::error::AppError;
use crate::models::User;

pub struct UserRepository {
    biz: String,
    pub(crate) pool: PgPool,
    pub(crate) rdb_pool: Pool<RedisConnectionManager>,
}

impl UserRepository {
    pub(crate) fn new(pool: PgPool, rdb_pool: Pool<RedisConnectionManager>) -> Self {
        Self {
            biz: "user".to_string(),
            pool,
            rdb_pool,
        }
    }

    pub(crate) async fn send_email_code(&self, email: &str) -> Result<(), AppError> {
        // generate a random 6-digit code
        let code = rand::random::<u32>() % 1000000;
        debug!("email code: {}", code);

        // save it in redis
        let mut rdb = self.rdb_pool.get()?;
        rdb.set_ex(format!("{}:{}:{}", self.biz, "email_code", email), code, 600)?;

        send_email_code(email, &code.to_string()).await?;

        Ok(())
    }

    pub(crate) async fn verify_email_code(&self, email: &str, code_input: &str) -> Result<bool, AppError> {
        let mut rdb = self.rdb_pool.get()?;
        let key = format!("{}:{}:{}", self.biz, "email_code", email);
        let code: Option<String> = rdb.get(key.clone())?;

        match code {
            Some(c) => {
                if c == code_input {
                    rdb.del(key)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(false)
        }
    }

    pub(crate) async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, fullname, email, created_at FROM users WHERE email = $1
            "#,
        )
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub(crate) async fn create(&self, email: &str, password: &str, fullname: &str) -> Result<User, AppError> {
        let user = self.find_by_email(email).await?;

        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(email.to_string()));
        }

        let password_hash = hash_password(password)?;

        let user = sqlx::query_as(
            r#"
            INSERT INTO users (email, fullname, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
            .bind(email.to_string())
            .bind(fullname.to_string())
            .bind(password_hash)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    pub(crate) async fn verify_password(&self, email: &str, password: &str) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, fullname, email, password_hash, created_at FROM users WHERE email = $1
            "#,
        )
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);

                let is_valid = verify_password(password, &password_hash.unwrap_or_default())?;

                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None)
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

    Ok(password_hash)
}

async fn send_email_code(email: &str, code: &str) -> Result<(), AppError> {
    let from = "863461783@qq.com".parse().unwrap();
    let to = email.parse().unwrap();
    let header = "text/html; charset=utf8".parse().unwrap();

    let message = MessageBuilder::new()
        .from(from)
        .to(to)
        .subject("iChat: Your Email Verification Code.")
        .header(lettre::message::header::ContentType::from(header))
        .body(format!("<h1>Your verification code is: {}</h1>", code))
        .map_err(|e| AppError::SmtpError(e.to_string()))?;

    let creds = Credentials::new("863461783@qq.com".to_string(), "ucqzmsgjeuqjbccf".to_string());

    let mailer = SmtpTransport::relay("smtp.qq.com")
        .map_err(|e| AppError::SmtpError(e.to_string()))?
        .credentials(creds)
        .build();

    mailer.send(&message).map_err(|e| AppError::SmtpError(e.to_string()))?;

    Ok(())
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    let is_valid = argon2.verify_password(password.as_bytes(), &password_hash).is_ok();

    Ok(is_valid)
}
