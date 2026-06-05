-- Ma'lumot uchun: baza tuzilishi (SQLite).
-- Buni qo'lda ishga tushirish SHART EMAS — ilova ishga tushganda
-- (db::init_db) jadvallarni o'zi yaratadi va namuna savollar qo'shadi.
-- Qo'lda ko'rmoqchi bo'lsangiz:  sqlite3 quiz.db < schema.sql

CREATE TABLE IF NOT EXISTS questions (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    question_text  TEXT NOT NULL,
    option_a       TEXT NOT NULL,
    option_b       TEXT NOT NULL,
    option_c       TEXT NOT NULL,
    option_d       TEXT NOT NULL,
    correct_option TEXT NOT NULL  -- 'a' | 'b' | 'c' | 'd'
);

CREATE TABLE IF NOT EXISTS users (
    telegram_id INTEGER PRIMARY KEY,
    username    TEXT    NOT NULL DEFAULT '',
    score       INTEGER NOT NULL DEFAULT 0,
    answered    INTEGER NOT NULL DEFAULT 0
);
