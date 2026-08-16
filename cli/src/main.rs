//! excalibur-ctl: Command-line control utility for Casper Excalibur gaming laptops.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

const LED_SYSFS_DIR: &str = "/sys/class/leds";
const HWMON_SYSFS_DIR: &str = "/sys/class/hwmon";
const PLATFORM_PROFILE_PATH: &str = "/sys/firmware/acpi/platform_profile";
const PLATFORM_PROFILE_CHOICES_PATH: &str = "/sys/firmware/acpi/platform_profile_choices";

const ZONES: &[(&str, &str)] = &[
    ("left", "excalibur::kbd_backlight-left"),
    ("middle", "excalibur::kbd_backlight-middle"),
    ("right", "excalibur::kbd_backlight-right"),
    ("corners", "excalibur::kbd_backlight-corners"),
];

fn print_banner() {
    println!("\x1b[1;36m╔════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("\x1b[1;36m║\x1b[0m   \x1b[1;37m⚔️  CASPER EXCALIBUR CONTROL CENTER (Rust CLI) ⚔️\x1b[0m        \x1b[1;36m║\x1b[0m");
    println!("\x1b[1;36m╚════════════════════════════════════════════════════════════╝\x1b[0m\n");
}

fn print_usage() {
    print_banner();
    println!("\x1b[1mKULLANIM:\x1b[0m");
    println!("  excalibur-ctl <KOMUT> [PARAMETRELER]\n");
    println!("\x1b[1mKOMUTLAR:\x1b[0m");
    println!("  \x1b[32mstatus\x1b[0m                          Sistem, fan, güç planı ve RGB durumunu gösterir");
    println!("  \x1b[32mprofile\x1b[0m [get|set <MOD>]         Güç planını okur veya değiştirir");
    println!("                                  Modlar: \x1b[33mperformance | gaming | quiet | low_power\x1b[0m (veya 1-4)");
    println!("  \x1b[32mrgb\x1b[0m [SEÇENEKLER]                Klavye RGB aydınlatmasını kontrol eder");
    println!("      --zone <left|middle|right|corners|all>   (Varsayılan: all)");
    println!("      --color <RRGGBB|#RRGGBB>                 (Örnek: FF0055, 00FFCC)");
    println!("      --mode <MOD>                             (Modlar: static, fade, blink, rainbow, heartbeat, wave, random, off)");
    println!("      --brightness <0..2>                      (0=Kapalı, 1=Düşük, 2=Yüksek)");
    println!("  \x1b[32mfan\x1b[0m [--watch]                   Fan devir hızlarını gösterir veya canlı izler\n");
    println!("\x1b[1mÖRNEKLER:\x1b[0m");
    println!("  excalibur-ctl status");
    println!("  excalibur-ctl profile set performance");
    println!("  excalibur-ctl rgb --zone all --color FF0033 --mode static --brightness 2");
    println!("  excalibur-ctl rgb --mode rainbow");
    println!("  excalibur-ctl fan --watch");
}

fn find_excalibur_hwmon() -> Option<PathBuf> {
    let entries = fs::read_dir(HWMON_SYSFS_DIR).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");
        if let Ok(name) = fs::read_to_string(name_path) {
            if name.trim() == "excalibur_wmi" {
                return Some(path);
            }
        }
    }
    None
}

fn get_fan_speeds() -> (Option<u32>, Option<u32>) {
    if let Some(hwmon) = find_excalibur_hwmon() {
        let cpu = fs::read_to_string(hwmon.join("fan1_input"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());
        let gpu = fs::read_to_string(hwmon.join("fan2_input"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok());
        (cpu, gpu)
    } else {
        (None, None)
    }
}

fn get_power_plan() -> (String, Option<u32>) {
    let platform_prof = fs::read_to_string(PLATFORM_PROFILE_PATH)
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let raw_pwm = if let Some(hwmon) = find_excalibur_hwmon() {
        fs::read_to_string(hwmon.join("pwm1"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
    } else {
        None
    };

    (platform_prof, raw_pwm)
}

fn cmd_status() {
    print_banner();

    // 1. Fan Speeds
    let (cpu_fan, gpu_fan) = get_fan_speeds();
    println!("\x1b[1;34m[+] FAN HIZLARI (HWMON)\x1b[0m");
    match cpu_fan {
        Some(rpm) => println!("  • CPU Fan : \x1b[1;32m{} RPM\x1b[0m", rpm),
        None => println!("  • CPU Fan : \x1b[31mOkunamadı\x1b[0m"),
    }
    match gpu_fan {
        Some(rpm) => println!("  • GPU Fan : \x1b[1;32m{} RPM\x1b[0m", rpm),
        None => println!("  • GPU Fan : \x1b[31mOkunamadı\x1b[0m"),
    }
    println!();

    // 2. Power Plan
    let (prof, pwm) = get_power_plan();
    let choices = fs::read_to_string(PLATFORM_PROFILE_CHOICES_PATH)
        .unwrap_or_else(|_| "performance, balanced, quiet, low-power".to_string())
        .trim()
        .to_string();

    println!("\x1b[1;34m[+] GÜÇ PLANI (POWER PROFILE)\x1b[0m");
    println!("  • Platform Profile : \x1b[1;33m{}\x1b[0m", prof);
    println!("  • Desteklenenler   : \x1b[36m{}\x1b[0m", choices);
    if let Some(p) = pwm {
        let plan_desc = match p {
            1 => "High Power / Turbo (1)",
            2 => "Gaming / Balanced (2)",
            3 => "Text Mode / Quiet (3)",
            4 => "Low Power / Eco (4)",
            _ => "Unknown",
        };
        println!("  • Donanım Modu     : \x1b[1;33m{}\x1b[0m", plan_desc);
    }
    println!();

    // 3. LED Status
    println!("\x1b[1;34m[+] KLAVYE VE KÖŞE RGB AYDINLATMA\x1b[0m");
    for (short_name, sysfs_name) in ZONES {
        let zone_dir = Path::new(LED_SYSFS_DIR).join(sysfs_name);
        if zone_dir.exists() {
            let brightness = fs::read_to_string(zone_dir.join("brightness"))
                .unwrap_or_else(|_| "0".into())
                .trim()
                .to_string();
            let max_brightness = fs::read_to_string(zone_dir.join("max_brightness"))
                .unwrap_or_else(|_| "2".into())
                .trim()
                .to_string();
            let mode = fs::read_to_string(zone_dir.join("mode"))
                .unwrap_or_else(|_| "unknown".into())
                .trim()
                .to_string();

            println!(
                "  • Bölge: \x1b[1m{:7}\x1b[0m | Parlaklık: \x1b[32m{}/{}\x1b[0m | Mod: \x1b[35m{}\x1b[0m",
                short_name, brightness, max_brightness, mode
            );
        } else {
            println!("  • Bölge: {:7} | \x1b[31mAygıt bulunamadı\x1b[0m", short_name);
        }
    }
    println!();
}

fn cmd_profile(args: &[String]) {
    if args.is_empty() || args[0] == "get" {
        let (prof, pwm) = get_power_plan();
        println!("Mevcut Güç Planı: \x1b[1;33m{}\x1b[0m (Donanım: {:?})", prof, pwm);
        return;
    }

    if args[0] == "set" {
        if args.len() < 2 {
            eprintln!("\x1b[31mHata: Ayarlanacak profil belirtilmedi.\x1b[0m");
            eprintln!("Kullanılabilir: performance, gaming, quiet, low_power (veya 1, 2, 3, 4)");
            exit(1);
        }

        let target = args[1].to_lowercase();
        let (platform_val, pwm_val) = match target.as_str() {
            "performance" | "high_power" | "turbo" | "1" => ("performance", "1"),
            "gaming" | "balanced" | "2" => ("balanced", "2"),
            "quiet" | "text_mode" | "silent" | "3" => ("quiet", "3"),
            "low_power" | "eco" | "battery" | "4" => ("low-power", "4"),
            _ => {
                eprintln!("\x1b[31mGeçersiz profil: {}\x1b[0m", target);
                exit(1);
            }
        };

        // Try setting via platform_profile first
        let pp_result = fs::write(PLATFORM_PROFILE_PATH, platform_val);
        if pp_result.is_err() {
            // Fallback to hwmon pwm1
            if let Some(hwmon) = find_excalibur_hwmon() {
                if let Err(e) = fs::write(hwmon.join("pwm1"), pwm_val) {
                    eprintln!("\x1b[31mGüç planı değiştirilemedi: {}\x1b[0m (Root yetkisi gerekebilir)", e);
                    exit(1);
                }
            } else {
                eprintln!("\x1b[31mPlatform profile ve HWMON arayüzü bulunamadı.\x1b[0m");
                exit(1);
            }
        }

        println!("\x1b[1;32m✓ Güç planı başarıyla ayarlandı: {}\x1b[0m", target);
        return;
    }

    eprintln!("\x1b[31mBilinmeyen profil alt komutu: {}\x1b[0m", args[0]);
}

fn cmd_rgb(args: &[String]) {
    let mut target_zone = "all".to_string();
    let mut target_color: Option<String> = None;
    let mut target_mode: Option<String> = None;
    let mut target_brightness: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--zone" | "-z" => {
                if i + 1 < args.len() {
                    target_zone = args[i + 1].to_lowercase();
                    i += 1;
                }
            }
            "--color" | "-c" => {
                if i + 1 < args.len() {
                    target_color = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--mode" | "-m" => {
                if i + 1 < args.len() {
                    target_mode = Some(args[i + 1].to_lowercase());
                    i += 1;
                }
            }
            "--brightness" | "-b" => {
                if i + 1 < args.len() {
                    target_brightness = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let zones_to_apply: Vec<&str> = if target_zone == "all" {
        vec!["left", "middle", "right", "corners"]
    } else {
        vec![target_zone.as_str()]
    };

    for zone in zones_to_apply {
        let sysfs_name = match zone {
            "left" => "excalibur::kbd_backlight-left",
            "middle" => "excalibur::kbd_backlight-middle",
            "right" => "excalibur::kbd_backlight-right",
            "corners" => "excalibur::kbd_backlight-corners",
            other => {
                eprintln!("\x1b[31mBilinmeyen bölge: {}\x1b[0m (left, middle, right, corners, all)", other);
                continue;
            }
        };

        let zone_dir = Path::new(LED_SYSFS_DIR).join(sysfs_name);
        if !zone_dir.exists() {
            eprintln!("\x1b[31mAygıt yolu mevcut değil: {:?}\x1b[0m", zone_dir);
            continue;
        }

        if let Some(ref color) = target_color {
            if let Err(e) = fs::write(zone_dir.join("color"), color) {
                eprintln!("\x1b[31mRenk yazılamadı ({}): {}\x1b[0m", zone, e);
            } else {
                println!("✓ [Zone: {:7}] Renk -> \x1b[33m#{}\x1b[0m", zone, color);
            }
        }

        if let Some(ref mode) = target_mode {
            if let Err(e) = fs::write(zone_dir.join("mode"), mode) {
                eprintln!("\x1b[31mMod yazılamadı ({}): {}\x1b[0m", zone, e);
            } else {
                println!("✓ [Zone: {:7}] Mod  -> \x1b[35m{}\x1b[0m", zone, mode);
            }
        }

        if let Some(ref br) = target_brightness {
            if let Err(e) = fs::write(zone_dir.join("brightness"), br) {
                eprintln!("\x1b[31mParlaklık yazılamadı ({}): {}\x1b[0m", zone, e);
            } else {
                println!("✓ [Zone: {:7}] Parlaklık -> \x1b[32m{}\x1b[0m", zone, br);
            }
        }
    }
}

fn cmd_fan(args: &[String]) {
    let watch = args.iter().any(|a| a == "--watch" || a == "-w");

    if !watch {
        let (cpu, gpu) = get_fan_speeds();
        println!("CPU Fan : \x1b[1;32m{} RPM\x1b[0m", cpu.unwrap_or(0));
        println!("GPU Fan : \x1b[1;32m{} RPM\x1b[0m", gpu.unwrap_or(0));
        return;
    }

    println!("\x1b[1;36mCanlı Fan Hızı İzleme (Çıkış için Ctrl+C)...\x1b[0m\n");
    loop {
        let (cpu, gpu) = get_fan_speeds();
        print!(
            "\r  CPU Fan: \x1b[1;32m{:4} RPM\x1b[0m  |  GPU Fan: \x1b[1;32m{:4} RPM\x1b[0m   ",
            cpu.unwrap_or(0),
            gpu.unwrap_or(0)
        );
        use std::io::Write;
        let _ = std::io::stdout().flush();
        sleep(Duration::from_millis(1000));
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        return;
    }

    match args[0].as_str() {
        "status" | "info" | "s" => cmd_status(),
        "profile" | "power" | "p" => cmd_profile(&args[1..]),
        "rgb" | "led" | "k" => cmd_rgb(&args[1..]),
        "fan" | "fans" | "f" => cmd_fan(&args[1..]),
        "help" | "-h" | "--help" => print_usage(),
        other => {
            eprintln!("\x1b[31mBilinmeyen komut: {}\x1b[0m\n", other);
            print_usage();
            exit(1);
        }
    }
}
