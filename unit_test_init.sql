-- Create User Table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) UNIQUE NOT NULL,
    password_hash VARCHAR(97) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Insert User
INSERT INTO users (fullname, email, password_hash) VALUES ('John Doe', '863461783@qq.com', '$argon2id$v=19$m=19456,t=2,p=1$yUvcv2ffMjquPxTKaheWGg$7kXDQl6Lf0FePxazRD0lvJMvsa7U4alrTp5HJmKTs/g');
