# Architecture · الخطة

## المراحل
| # | المحتوى |
|---|---------|
| 1 | Steam (حسابين أو أكثر) + Epic + Rockstar: فحص، مكتبة موحّدة، تبديل حساب ستيم تلقائي، تشغيل |
| 2 | GOG، EA app، Ubisoft Connect، Battle.net، Xbox + أغلفة رسمية (SteamGridDB) |
| 3 | المودات لأول لعبتين/ثلاث: تصفح، تثبيت، تفعيل/تعطيل، حذف نظيف |
| 4 | توسيع دعم المودات لألعاب أكثر |

## تعدد حسابات ستيم (المرحلة ١)
ستيم يسمح بحساب واحد مسجّل في نفس الوقت. المستخدم يملك ألعاباً على حسابين، و**Call of Duty على الحساب الثاني لأن الحساب الأول محظور فيها**، لذلك مشاركة العائلة (Steam Families) غير مناسبة: لازم اللعبة تشتغل من الحساب المالك لها.

عند تشغيل لعبة مالكها ليس الحساب النشط:
1. إغلاق ستيم (`steam.exe -shutdown`) وانتظار خروجه.
2. ضبط `HKCU\Software\Valve\Steam\AutoLoginUser` على اسم الحساب، وتعليمه `MostRecent` في `config/loginusers.vdf`.
3. تشغيل ستيم (يدخل تلقائياً بالجلسة المحفوظة).
4. `steam://rungameid/<appid>`.

- الشرط: كل حساب سُجّل دخوله مرة بخيار «تذكرني».
- لا تُقرأ ولا تُحفظ أي كلمة مرور.
- معرفة ألعاب كل حساب: Steam Web API `GetOwnedGames` لكل SteamID، مع الملفات المحلية للألعاب المثبتة.
- قاعدة أمان: لا تشغيل أبداً لـ Call of Duty على الحساب الأول (قاعدة «حساب مفروض لكل لعبة» قابلة للتعديل من الإعدادات).

## المنصات
| المنصة | الاكتشاف | التشغيل |
|---|---|---|
| Steam | `libraryfolders.vdf` + `appmanifest_*.acf`، `loginusers.vdf` | `steam://rungameid/ID` |
| Epic | `ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` | `com.epicgames.launcher://apps/...?action=launch` |
| Rockstar | Registry `HKLM\SOFTWARE\WOW6432Node\Rockstar Games` | Rockstar Launcher |
| GOG | Registry + قاعدة GOG Galaxy | ملف اللعبة مباشرة |
| EA / Ubisoft / Battle.net | Registry + ملفات المنصة | روابط المنصة |
| Xbox | حزم التطبيقات (AppX) | `shell:AppsFolder` |

## المودات (المرحلة ٣)
- المصادر: mod.io وThunderstore (تنزيل مباشر)، Nexus Mods (مفتاح API للمستخدم؛ الحساب المجاني عبر زر Mod Manager Download → رابط `nxm://` يستقبله ساحة).
- التثبيت: المودات تُحفظ في مجلد تجهيز وتُربط بملفات اللعبة، فالتعطيل والحذف يرجّعان الملفات كما كانت.
- قواعد خاصة لكل لعبة: مجلد الهدف، المحمّل المطلوب (ScriptHookV، Cyber Engine Tweaks، ModEngine2…)، ترتيب التحميل.
- الألعاب الأونلاين (GTA Online، Call of Duty، CS2، Fortnite): تُعلَّم وتُمنع المودات فيها لتجنب الحظر.
