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

    pub(crate) async fn send_email_code(&self, email: &str) -> Result<String, AppError> {
        // generate a random 6-digit code
        let code = rand::random::<u32>() % 1000000;
        // pad it to 6 digits
        let code = format!("{:06}", code);

        debug!("email code: {}", code);

        // save it in redis
        let mut rdb = self.rdb_pool.get()?;
        rdb.set_ex(format!("{}:{}:{}", self.biz, "email_code", email), code.clone(), 600)?;

        send_email_code(email, &code.to_string()).await?;

        Ok(code)
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

    pub(crate) async fn find_by_id(&self, id: i64) -> Result<Option<User>, AppError> {
        let user: Option<User> = sqlx::query_as(
            r#"
            SELECT id, fullname, email, created_at FROM users WHERE id = $1
            "#,
        )
            .bind(id)
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

    pub(crate) async fn verify_password(&self, email: &str, password: &str) -> Result<User, AppError> {
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
                    Ok(user)
                } else {
                    Err(AppError::PasswordError)
                }
            }
            None => Err(AppError::UserNotFound),
        }
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)?;

    let is_valid = argon2.verify_password(password.as_bytes(), &password_hash).is_ok();

    Ok(is_valid)
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


#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use super::*;

    #[tokio::test]
    async fn send_email_code_should_work() {
        send_email_code("863461783@qq.com", "123456").await.unwrap();
    }

    #[test]
    fn hash_password_should_work() {
        let password = "password";

        let password_hash = hash_password(password).unwrap();

        assert_ne!(password, password_hash);
    }

    #[test]
    fn verify_password_should_work() {
        let password = "password";
        let password_hash = hash_password(password).unwrap();

        let is_valid = verify_password(password, &password_hash).unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn user_repo_send_email_code_and_verify_should_work() {
        let config = AppConfig::load().unwrap();

        let pool = PgPool::connect(config.server.postgres_url.as_str())
            .await
            .unwrap();

        let redis_manager = RedisConnectionManager::new(config.server.redis_url.as_str())
            .expect("Failed to create redis connection manager");

        let rdb_pool = Pool::builder().max_size(15).build(redis_manager)
            .expect("Failed to create redis pool");

        let repo = UserRepository::new(pool, rdb_pool.clone());

        let code = repo.send_email_code("863461783@qq.com").await.unwrap();

        let is_valid = repo.verify_email_code("863461783@qq.com", "123456").await.unwrap();
        assert!(!is_valid);

        let is_valid = repo.verify_email_code("863461783@qq.com", &code).await.unwrap();
        assert!(is_valid);

        let is_valid = repo.verify_email_code("123@qq.com", &code).await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn user_repo_find_by_email_should_work() {
        let config = AppConfig::load().unwrap();

        let pool = PgPool::connect(config.server.postgres_url.as_str())
            .await
            .unwrap();

        let redis_manager = RedisConnectionManager::new(config.server.redis_url.as_str())
            .expect("Failed to create redis connection manager");

        let rdb_pool = Pool::builder().max_size(15).build(redis_manager)
            .expect("Failed to create redis pool");

        let repo = UserRepository::new(pool, rdb_pool.clone());

        // TODO: prepare a test database with init data
        let user = repo.find_by_email("863461783@qq.com").await.unwrap();
        assert_eq!(user.unwrap().email, "863461783@qq.com");

        let user = repo.find_by_email("1").await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn user_repo_create_user_should_work() {
        let config = AppConfig::load().unwrap();

        let pool = PgPool::connect(config.server.postgres_url.as_str())
            .await
            .unwrap();

        let redis_manager = RedisConnectionManager::new(config.server.redis_url.as_str())
            .expect("Failed to create redis connection manager");

        let rdb_pool = Pool::builder().max_size(15).build(redis_manager)
            .expect("Failed to create redis pool");

        let repo = UserRepository::new(pool, rdb_pool.clone());

        // cannot insert the same email
        let user = repo.create("863461783@qq.com", "123456", "bobo").await;
        assert!(user.is_err());

        let user = repo.create("unit_test@qq.com", "123456", "bobo").await;
        assert!(user.is_ok());
    }

    #[tokio::test]
    async fn user_repo_verify_password_should_work() {
        let config = AppConfig::load().unwrap();

        let pool = PgPool::connect(config.server.postgres_url.as_str())
            .await
            .unwrap();

        let redis_manager = RedisConnectionManager::new(config.server.redis_url.as_str())
            .expect("Failed to create redis connection manager");

        let rdb_pool = Pool::builder().max_size(15).build(redis_manager)
            .expect("Failed to create redis pool");

        let repo = UserRepository::new(pool, rdb_pool.clone());

        let user = repo.verify_password("863461783@qq.com", "123456").await;
        assert!(user.is_ok());

        let user = repo.verify_password("not_exist_user@qq.com", "123456").await;
        assert!(user.is_err());

        let user = repo.verify_password("863461783@qq.com", "123789").await;
        assert!(user.is_err());
    }
}
