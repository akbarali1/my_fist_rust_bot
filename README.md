# 🧠 Quiz Bot — Telegram test boti

Rustda yozilgan oddiy Telegram **test/quiz** boti. Foydalanuvchiga tasodifiy
savol beradi, javobni tekshiradi, ball to'playdi va reyting ko'rsatadi.
Ma'lumotlar **SQLite** bazasida saqlanadi.

> Bu — Rust bilan tanishuv uchun yozilgan ilk loyiha. Kodning har bir qismi
> izohlar bilan tushuntirilgan.

## ✨ Imkoniyatlar

- `/start` — botni ishga tushirish va salomlashish
- `/quiz` — tasodifiy savol (4 ta variant, inline tugmalar bilan)
- `/score` — sizning ballaringiz va javoblar soni
- `/top` — eng yuqori 10 ta foydalanuvchi (reyting)
- Javob bosilganda darhol ✅ / ❌ ko'rsatadi va ball bazaga yoziladi

## 🛠 Texnologiyalar

| Vazifa | Kutubxona |
|--------|-----------|
| Telegram bot | [teloxide](https://github.com/teloxide/teloxide) |
| Async runtime | [tokio](https://tokio.rs) |
| Baza (SQLite) | [sqlx](https://github.com/launchbadge/sqlx) |
| `.env` o'qish | [dotenvy](https://github.com/allan2/dotenvy) |

Yangilanish usuli: **long polling** (webhook emas) — server yoki domen kerak emas.

## 📋 Talablar

- [Rust](https://rustup.rs) 1.88 yoki undan yuqori (eng yangi `stable` tavsiya etiladi: `rustup update stable`)
- Telegram bot tokeni — [@BotFather](https://t.me/BotFather) dan olinadi

## 🚀 Ishga tushirish

```bash
# 1. Repozitoriyni klonlash
git clone https://github.com/akbarali1/my_fist_rust_bot.git
cd my_fist_rust_bot

# 2. .env faylini yaratish
cp .env.example .env
```

So'ng `.env` faylini ochib, `TELOXIDE_TOKEN` ga @BotFather dan olgan
tokeningizni qo'ying:

```env
TELOXIDE_TOKEN=123456789:AAAA-bbbbCCCC...
DATABASE_URL=sqlite:quiz.db
```

```bash
# 3. Ishga tushirish (baza va jadvallar avtomatik yaratiladi)
cargo run
```

To'xtatish uchun **Ctrl+C**. So'ng Telegram'da botingizga `/start` yozing.

## 📁 Loyiha tuzilishi

```
my_fist_rust_bot/
├── Cargo.toml      # bog'liqliklar (dependencies)
├── schema.sql      # baza tuzilishi (ma'lumot uchun)
├── .env.example    # sozlamalar namunasi
└── src/
    ├── main.rs     # bot logikasi: komandalar va tugmalar
    └── db.rs       # SQLite funksiyalari va init_db()
```

## 🗄 Ma'lumotlar bazasi

Bot birinchi marta ishga tushganda `quiz.db` fayli va kerakli jadvallar
(`questions`, `users`) **avtomatik yaratiladi**, hamda 5 ta namuna savol
qo'shiladi. Hech narsani qo'lda sozlash shart emas.

## 💡 Keyingi g'oyalar

- Admin uchun `/addquiz` — yangi savol qo'shish
- Savol kategoriyalari yoki qiyinlik darajalari
- Bir savolga ikki marta javob berishni bloklash
- Production uchun webhook'ga o'tish

## 📄 Litsenziya

MIT
