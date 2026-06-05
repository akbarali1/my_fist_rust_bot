//! Quiz/Test Telegram bot — Rust + teloxide + MySQL (sqlx).

mod db;

use db::Db;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

/// Handler'lar shu turdagi natija qaytaradi.
/// `?` operatori orqali ham teloxide, ham sqlx xatolarini ushlay oladi.
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Bot komandalari (Telegram menyusida ko'rinadi).
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Mavjud komandalar:")]
enum Command {
    #[command(description = "botni ishga tushirish")]
    Start,
    #[command(description = "yangi savol olish")]
    Quiz,
    #[command(description = "ballaringizni ko'rish")]
    Score,
    #[command(description = "reyting (top 10)")]
    Top,
    #[command(description = "yordam")]
    Help,
}

#[tokio::main]
async fn main() -> HandlerResult {
    dotenvy::dotenv().ok();

    // .env da ko'rsatilmasa, joriy papkada quiz.db faylini ishlatamiz.
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:quiz.db".to_string());

    let pool = db::connect(&database_url).await?;
    db::init_db(&pool).await?; // jadvallarni yaratadi va namuna savollar qo'shadi
    println!("✅ Bazaga ulandik ({database_url})");

    // TELOXIDE_TOKEN muhit o'zgaruvchisidan o'qiydi.
    let bot = Bot::from_env();
    bot.set_my_commands(Command::bot_commands()).await?;
    println!("🤖 Bot ishga tushdi. To'xtatish uchun Ctrl+C bosing.");

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(answer_command),
        )
        .branch(Update::filter_callback_query().endpoint(answer_callback));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

/// Matnli komandalarni qayta ishlaydi (/start, /quiz, ...).
async fn answer_command(bot: Bot, msg: Message, cmd: Command, pool: Db) -> HandlerResult {
    match cmd {
        Command::Start | Command::Help => {
            if let Some(user) = msg.from.as_ref() {
                let name = user
                    .username
                    .clone()
                    .unwrap_or_else(|| user.first_name.clone());
                db::register_user(&pool, user.id.0 as i64, &name).await?;
            }
            bot.send_message(
                msg.chat.id,
                "👋 Salom! Bu — Quiz bot.\n\n\
                 /quiz — yangi savol\n\
                 /score — ballaringiz\n\
                 /top — reyting (top 10)",
            )
            .await?;
        }
        Command::Quiz => match db::random_question(&pool).await? {
            Some(q) => {
                bot.send_message(msg.chat.id, format!("❓ {}", q.question_text))
                    .reply_markup(quiz_keyboard(&q))
                    .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Hozircha savollar yo'q 😕")
                    .await?;
            }
        },
        Command::Score => {
            let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
            let (score, answered) = db::get_score(&pool, user_id).await?;
            bot.send_message(
                msg.chat.id,
                format!("📊 Ballaringiz: {score}\nJavob berilgan: {answered} ta"),
            )
            .await?;
        }
        Command::Top => {
            let rows = db::leaderboard(&pool).await?;
            if rows.is_empty() {
                bot.send_message(msg.chat.id, "Reyting hozircha bo'sh.")
                    .await?;
            } else {
                let mut text = String::from("🏆 Reyting (Top 10):\n\n");
                for (i, (name, score)) in rows.iter().enumerate() {
                    text.push_str(&format!("{}. {} — {} ball\n", i + 1, name, score));
                }
                bot.send_message(msg.chat.id, text).await?;
            }
        }
    }
    Ok(())
}

/// Inline tugma bosilganda javobni tekshiradi.
async fn answer_callback(bot: Bot, q: CallbackQuery, pool: Db) -> HandlerResult {
    // Telegram'ga "qabul qilindi" signalini yuboramiz (soat aylanmasligi uchun).
    bot.answer_callback_query(q.id.clone()).await?;

    // callback_data formati: "a|12" (tanlangan harf | savol id).
    let Some(data) = q.data.as_deref() else {
        return Ok(());
    };
    let mut parts = data.split('|');
    let chosen = parts.next().unwrap_or("");
    let qid: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    let Some(question) = db::question_by_id(&pool, qid).await? else {
        return Ok(());
    };

    let correct = question.correct_option.eq_ignore_ascii_case(chosen);
    let user_id = q.from.id.0 as i64;
    let name = q
        .from
        .username
        .clone()
        .unwrap_or_else(|| q.from.first_name.clone());
    db::register_user(&pool, user_id, &name).await?;
    db::record_answer(&pool, user_id, correct).await?;

    let verdict = if correct {
        "✅ To'g'ri javob!".to_string()
    } else {
        format!(
            "❌ Noto'g'ri. To'g'ri javob: {}",
            question.correct_option.to_uppercase()
        )
    };

    // Savol xabarini natija bilan yangilaymiz (tugmalar o'rniga).
    if let Some(message) = q.message {
        let text = format!(
            "❓ {}\n\n{}\n\nYana savol uchun: /quiz",
            question.question_text, verdict
        );
        bot.edit_message_text(message.chat().id, message.id(), text)
            .await?;
    }
    Ok(())
}

/// Savol uchun 4 ta variantli inline klaviatura yasaydi.
fn quiz_keyboard(q: &db::Question) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            format!("A) {}", q.option_a),
            format!("a|{}", q.id),
        )],
        vec![InlineKeyboardButton::callback(
            format!("B) {}", q.option_b),
            format!("b|{}", q.id),
        )],
        vec![InlineKeyboardButton::callback(
            format!("C) {}", q.option_c),
            format!("c|{}", q.id),
        )],
        vec![InlineKeyboardButton::callback(
            format!("D) {}", q.option_d),
            format!("d|{}", q.id),
        )],
    ])
}
