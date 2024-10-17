-- Create User Table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) UNIQUE NOT NULL,
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create Chat Types
CREATE TYPE chat_type AS ENUM ('private', 'group');

-- Create Chat Table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    type chat_type NOT NULL,
    name VARCHAR(64),
    owner_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create Chat Members Table
CREATE TABLE IF NOT EXISTS chat_members (
    chat_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (chat_id, user_id),
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create Message Table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);


-- Some initial data
INSERT INTO "public"."users" ("id", "fullname", "email", "password_hash", "created_at") VALUES
(1, '小李', '863461789@qq.com', '$argon2id$v=19$m=19456,t=2,p=1$VC5FyXCFoBj24OiecgkUPg$V7pR9gPHcxGka8AujfTudTdKn7UtlY6OJ52LmxStskI', '2024-10-16 11:49:39.417957+00'),
(2, '小张', '863461710@qq.com', '$argon2id$v=19$m=19456,t=2,p=1$VC5FyXCFoBj24OiecgkUPg$V7pR9gPHcxGka8AujfTudTdKn7UtlY6OJ52LmxStskI', '2024-10-16 11:49:39.417957+00'),
(3, '小李', '863461711@qq.com', '$argon2id$v=19$m=19456,t=2,p=1$VC5FyXCFoBj24OiecgkUPg$V7pR9gPHcxGka8AujfTudTdKn7UtlY6OJ52LmxStskI', '2024-10-16 11:49:39.417957+00');
