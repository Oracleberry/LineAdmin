-- Users table: LINE user information
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_user_id TEXT NOT NULL UNIQUE,
    display_name TEXT,
    picture_url TEXT,
    status_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Messages table: Received messages from LINE users
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_user_id TEXT NOT NULL,
    message_type TEXT NOT NULL, -- text, image, video, audio, location, sticker
    message_text TEXT,
    message_data TEXT, -- JSON data for non-text messages
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (line_user_id) REFERENCES users(line_user_id)
);

-- Scheduled messages table: Messages to be sent at specific times
CREATE TABLE IF NOT EXISTS scheduled_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_user_id TEXT, -- NULL for broadcast messages
    message_text TEXT NOT NULL,
    schedule_time DATETIME NOT NULL,
    cron_expression TEXT, -- For recurring messages
    status TEXT DEFAULT 'pending', -- pending, sent, failed, cancelled
    sent_at DATETIME,
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (line_user_id) REFERENCES users(line_user_id)
);

-- Settings table: Application settings and API keys
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Calendars table: User calendar events
CREATE TABLE IF NOT EXISTS calendars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    line_user_id TEXT NOT NULL,
    event_title TEXT NOT NULL,
    event_description TEXT,
    event_time DATETIME NOT NULL,
    reminder_sent BOOLEAN DEFAULT FALSE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (line_user_id) REFERENCES users(line_user_id)
);

-- Notifications log table: Track sent notifications
CREATE TABLE IF NOT EXISTS notification_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    notification_type TEXT NOT NULL, -- line_notify, slack
    recipient TEXT NOT NULL,
    message TEXT NOT NULL,
    status TEXT NOT NULL, -- success, failed
    error_message TEXT,
    sent_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_messages_line_user_id ON messages(line_user_id);
CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);
CREATE INDEX IF NOT EXISTS idx_scheduled_messages_status ON scheduled_messages(status);
CREATE INDEX IF NOT EXISTS idx_scheduled_messages_schedule_time ON scheduled_messages(schedule_time);
CREATE INDEX IF NOT EXISTS idx_calendars_line_user_id ON calendars(line_user_id);
CREATE INDEX IF NOT EXISTS idx_calendars_event_time ON calendars(event_time);
