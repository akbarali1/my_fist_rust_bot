//! SQLite bilan ishlash uchun barcha funksiyalar shu yerda.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;

/// Ulanishlar puli (connection pool) uchun qisqa nom.
pub type Db = Pool<Sqlite>;

/// `questions` jadvalidagi bitta savol.
#[derive(Debug, sqlx::FromRow)]
pub struct Question {
    pub id: i64,
    pub question_text: String,
    pub option_a: String,
    pub option_b: String,
    pub option_c: String,
    pub option_d: String,
    /// To'g'ri javob harfi: 'a', 'b', 'c' yoki 'd'.
    pub correct_option: String,
}

/// SQLite faylga ulanadi (fayl bo'lmasa, o'zi yaratadi).
pub async fn connect(url: &str) -> Result<Db, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

/// Jadvallarni yaratadi va (bo'sh bo'lsa) namuna savollar qo'shadi.
pub async fn init_db(pool: &Db) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS questions (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            question_text  TEXT NOT NULL,
            option_a       TEXT NOT NULL,
            option_b       TEXT NOT NULL,
            option_c       TEXT NOT NULL,
            option_d       TEXT NOT NULL,
            correct_option TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            telegram_id INTEGER PRIMARY KEY,
            username    TEXT    NOT NULL DEFAULT '',
            score       INTEGER NOT NULL DEFAULT 0,
            answered    INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(pool)
    .await?;

    // Faqat jadval bo'sh bo'lsa namuna savollarni qo'shamiz.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM questions")
        .fetch_one(pool)
        .await?;
    if count == 0 {
        let samples = [
            ("Rust tilining 1.0 versiyasi qaysi yilda chiqqan?", "2010", "2015", "2018", "2020", "b"),
            ("Rustda o'zgaruvchi standart holatda qanday?", "mutable", "immutable", "static", "global", "b"),
            ("Cargo nima vazifani bajaradi?", "Faqat test", "Paket menejeri va build tizimi", "Matn muharriri", "Brauzer", "b"),
            ("Rustda xotira xavfsizligini nima ta'minlaydi?", "Garbage Collector", "Ownership tizimi", "Qo'lda free()", "Hech narsa", "b"),
            ("Qaysi biri Rust funksiya kalit so'zi?", "function", "def", "fn", "func", "c"),
        ];
        for (q, a, b, c, d, correct) in samples {
            sqlx::query(
                "INSERT INTO questions
                 (question_text, option_a, option_b, option_c, option_d, correct_option)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(q)
            .bind(a)
            .bind(b)
            .bind(c)
            .bind(d)
            .bind(correct)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

/// Tasodifiy bitta savol qaytaradi (savol bo'lmasa `None`).
pub async fn random_question(pool: &Db) -> Result<Option<Question>, sqlx::Error> {
    sqlx::query_as::<_, Question>(
        "SELECT id, question_text, option_a, option_b, option_c, option_d, correct_option \
         FROM questions ORDER BY RANDOM() LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

/// `id` bo'yicha savolni topadi.
pub async fn question_by_id(pool: &Db, id: i64) -> Result<Option<Question>, sqlx::Error> {
    sqlx::query_as::<_, Question>(
        "SELECT id, question_text, option_a, option_b, option_c, option_d, correct_option \
         FROM questions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Foydalanuvchini ro'yxatga qo'shadi yoki ismini yangilaydi.
pub async fn register_user(pool: &Db, telegram_id: i64, username: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO users (telegram_id, username) VALUES (?, ?) \
         ON CONFLICT(telegram_id) DO UPDATE SET username = excluded.username",
    )
    .bind(telegram_id)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

/// Foydalanuvchining javobini hisobga oladi.
/// To'g'ri bo'lsa `score` +1, har holatda `answered` +1.
pub async fn record_answer(pool: &Db, telegram_id: i64, correct: bool) -> Result<(), sqlx::Error> {
    let inc: i64 = if correct { 1 } else { 0 };
    sqlx::query(
        "INSERT INTO users (telegram_id, score, answered) VALUES (?, ?, 1) \
         ON CONFLICT(telegram_id) DO UPDATE SET score = score + ?, answered = answered + 1",
    )
    .bind(telegram_id)
    .bind(inc)
    .bind(inc)
    .execute(pool)
    .await?;
    Ok(())
}

/// (ball, jami javoblar) qaytaradi. Foydalanuvchi yo'q bo'lsa (0, 0).
pub async fn get_score(pool: &Db, telegram_id: i64) -> Result<(i64, i64), sqlx::Error> {
    let row: Option<(i64, i64)> =
        sqlx::query_as("SELECT score, answered FROM users WHERE telegram_id = ?")
            .bind(telegram_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.unwrap_or((0, 0)))
}

/// Eng yuqori ballga ega 10 ta foydalanuvchi: (ism, ball).
pub async fn leaderboard(pool: &Db) -> Result<Vec<(String, i64)>, sqlx::Error> {
    sqlx::query_as("SELECT username, score FROM users ORDER BY score DESC LIMIT 10")
        .fetch_all(pool)
        .await
}
