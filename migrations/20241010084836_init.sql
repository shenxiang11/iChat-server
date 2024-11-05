-- Create User Table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) UNIQUE NOT NULL,
    password_hash VARCHAR(97) NOT NULL,
    avatar VARCHAR(256),
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
    unread_count INT DEFAULT 0,
    PRIMARY KEY (chat_id, user_id),
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create Message Type
CREATE TYPE message_type AS ENUM ('text', 'image', 'video', 'audio', 'file');

-- Create Message Table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL,
    chat_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    type message_type NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (chat_id, id),
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
) PARTITION BY LIST (chat_id);

-- if chat changed, notify with chat data
CREATE OR REPLACE FUNCTION notify_chat_change()
    RETURNS TRIGGER
    AS $$
BEGIN
    RAISE NOTICE 'Chat changed: %', NEW;
    PERFORM pg_notify('chat_change', json_build_object('op', TG_OP, 'old', OLD, 'new', NEW)::text);
    RETURN NEW;
END;
    $$
LANGUAGE plpgsql;

CREATE TRIGGER chat_change_trigger
    AFTER INSERT OR UPDATE OR DELETE
    ON chats
    FOR EACH ROW
    EXECUTE FUNCTION notify_chat_change();

CREATE OR REPLACE FUNCTION increase_unread_count(chat_id BIGINT, user_id BIGINT)
    RETURNS VOID
    AS $$
BEGIN
    UPDATE chat_members AS t
        SET t.unread_count = t.unread_count + 1
        WHERE chat_id = t.chat_id AND user_id != t.user_id;
END;
$$ LANGUAGE plpgsql;

-- if some user send a message, notify this message to all chat members
CREATE OR REPLACE FUNCTION notify_message()
    RETURNS TRIGGER
    AS $$
BEGIN
    PERFORM increase_unread_count(NEW.chat_id, NEW.user_id);
    PERFORM pg_notify('new_message', row_to_json(NEW)::text);
    RETURN NEW;
END;
    $$
LANGUAGE plpgsql;


CREATE TRIGGER message_insert_trigger
    AFTER INSERT
    ON messages
    FOR EACH ROW
    EXECUTE FUNCTION notify_message();

