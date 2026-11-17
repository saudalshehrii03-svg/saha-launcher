//! Xbox / Microsoft Store (PC Game Pass).
//! - installed games: AppX packages that ship a MicrosoftGame.config (GDK games)
//!   listed through PowerShell; title from MicrosoftGame.config, app id from AppxManifest.xml
//! - launch: shell:AppsFolder\<PackageFamilyName>!<AppId>
use serde::Deserialize;

use crate::error::Result;
use crate::library::models::{Game, PlatformId};
use crate::sys::{self, Hive};

use super::{Platform, Progress};

pub struct Xbox;

const SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$out = @()
foreach ($p in Get-AppxPackage) {
  if (-not $p.InstallLocation) { continue }
  $cfg = Join-Path $p.InstallLocation 'MicrosoftGame.config'
  if (-not (Test-Path $cfg)) { continue }
  [xml]$g = Get-Content $cfg
  [xml]$m = Get-Content (Join-Path $p.InstallLocation 'AppxManifest.xml')
  $app = @($m.Package.Applications.Application)[0].Id
  $name = $g.Game.ShellVisuals.DefaultDisplayName
  if (-not $name) { $name = $m.Package.Properties.DisplayName }
  $out += [pscustomobject]@{ name = "$name"; pfn = $p.PackageFamilyName; app = "$app"; dir = $p.InstallLocation }
}
ConvertTo-Json -InputObject @($out) -Compress
"#;

#[derive(Deserialize)]
struct Pkg {
    name: String,
    pfn: String,
    app: String,
    dir: String,
}

impl Platform for Xbox {
    fn id(&self) -> PlatformId {
        PlatformId::Xbox
    }
    fn name(&self) -> &'static str {
        "Xbox"
    }
    fn install_path(&self) -> Option<String> {
        // Gaming Services is what installs PC Game Pass / Xbox games.
        sys::reg_str(Hive::LocalMachine, r"SYSTEM\CurrentControlSet\Services\GamingServices", "ImagePath")
            .map(|_| "Microsoft Store · Xbox".into())
    }
    fn scan_games(&self) -> Result<Vec<Game>> {
        let out = sys::hidden("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
            .output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let pkgs: Vec<Pkg> = serde_json::from_str(text.trim()).unwrap_or_default();
        Ok(pkgs
            .into_iter()
            .filter(|p| !p.app.is_empty())
            .map(|p| {
                let app_id = format!("{}!{}", p.pfn, p.app);
                Game::installed(PlatformId::Xbox, &p.pfn, &p.name, &app_id, Some(p.dir))
            })
            .collect())
    }
    fn launch(&self, game: &Game, progress: Progress) -> Result<()> {
        progress("تشغيل اللعبة…");
        sys::hidden("explorer.exe").arg(format!(r"shell:AppsFolder\{}", game.app_id)).spawn()?;
        Ok(())
    }
}
