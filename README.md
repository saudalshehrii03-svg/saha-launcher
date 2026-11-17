# ساحة · Saha Launcher

لانشر واحد يغنيك عن باقي اللانشرات: يجمع ألعابك من Steam وEpic وRockstar وغيرها، يشغّل كل لعبة من الحساب اللي يملكها بدون تبديل يدوي، ويثبّت المودات تلقائياً من داخل صفحة اللعبة.

One launcher for every library: merges Steam, Epic, Rockstar and more, launches each game on the account that owns it (automatic Steam account switching), and installs mods in-app.

> الحالة: أساس المشروع فقط. لا توجد ميزات منفّذة بعد. انظر [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## التقنية · Stack
- Tauri 2 (Rust) للخلفية: فحص المنصات، تبديل الحسابات، تثبيت المودات
- React 19 + TypeScript + Vite للواجهة
- الهوية البصرية: انظر `CLAUDE.md` وصور `docs/design/`
- الهدف: Windows (حزمة NSIS)

## التشغيل · Run
```
npm install
npm run tauri dev     # تطوير
npm run tauri build   # ملف التثبيت
```
المتطلبات: Node 20+، Rust stable، وعلى ويندوز: Visual Studio Build Tools وWebView2.

## الهيكل · Layout
```
src/                    الواجهة (React)
  pages/                المكتبة، المودات، المنصات والحسابات، الإعدادات
  lib/api.ts            استدعاءات Tauri
  lib/types.ts          نماذج البيانات (مطابقة لـ Rust)
  styles/tokens.css     ألوان وهوية التصميم
src-tauri/src/
  platforms/            ماسح ومشغّل لكل منصة (steam, epic, rockstar, ...)
  accounts/             تعدد الحسابات والتبديل التلقائي في ستيم
  mods/                 مصادر المودات وقواعد التثبيت لكل لعبة
  library/models.rs     Game, Account, PlatformStatus
  commands.rs           الأوامر المتاحة للواجهة
```
