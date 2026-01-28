use crate::db::user::User;
use crate::terminal::{AnsiWriter, Color};

/// Format a duration in minutes to a human-readable string.
///
/// Examples:
///   0 -> "0m"
///   45 -> "45m"
///   90 -> "1h 30m"
///   1440 -> "24h 0m"
pub fn format_duration_minutes(minutes: i64) -> String {
    if minutes < 60 {
        format!("{}m", minutes)
    } else {
        let hours = minutes / 60;
        let mins = minutes % 60;
        format!("{}h {}m", hours, mins)
    }
}

/// Format an ISO datetime string as "Month Day, Year" (e.g., "January 26, 2026").
///
/// Falls back to the raw string if parsing fails.
fn format_date(iso: &str) -> String {
    // Parse "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DDTHH:MM:SS" format
    let date_part = if iso.len() >= 10 { &iso[..10] } else { iso };
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() == 3 {
        let year = parts[0];
        let month = match parts[1] {
            "01" => "January",
            "02" => "February",
            "03" => "March",
            "04" => "April",
            "05" => "May",
            "06" => "June",
            "07" => "July",
            "08" => "August",
            "09" => "September",
            "10" => "October",
            "11" => "November",
            "12" => "December",
            _ => return iso.to_string(),
        };
        let day = parts[2].trim_start_matches('0');
        if day.is_empty() {
            return iso.to_string();
        }
        format!("{} {}, {}", month, day, year)
    } else {
        iso.to_string()
    }
}

/// Format an ISO datetime string as "Month Day, Year at H:MM AM/PM".
///
/// Falls back to date-only format if time parsing fails.
fn format_datetime(iso: &str) -> String {
    let date_str = format_date(iso);

    // Try to extract time portion after space or 'T'
    let time_part = if iso.len() > 11 {
        let sep = if iso.contains('T') { 'T' } else { ' ' };
        iso.split(sep).nth(1)
    } else {
        None
    };

    if let Some(time) = time_part {
        let time_parts: Vec<&str> = time.split(':').collect();
        if time_parts.len() >= 2 {
            if let (Ok(hour), Ok(min)) = (
                time_parts[0].parse::<u32>(),
                time_parts[1].parse::<u32>(),
            ) {
                let (h12, ampm) = if hour == 0 {
                    (12, "AM")
                } else if hour < 12 {
                    (hour, "AM")
                } else if hour == 12 {
                    (12, "PM")
                } else {
                    (hour - 12, "PM")
                };
                return format!("{} at {}:{:02} {}", date_str, h12, min, ampm);
            }
        }
    }

    date_str
}

/// Render a user profile as an ANSI art card with CP437 double-line box-drawing.
///
/// The card is 80 characters wide and uses CGA colors:
/// - Border: LightCyan
/// - Section headers: Yellow, bold
/// - Field labels: LightGray
/// - Field values: White, bold
/// - Level badge: LightMagenta for Sysop, LightGreen for User
pub fn render_profile_card(user: &User, _is_own_profile: bool) -> String {
    let mut w = AnsiWriter::new();

    let border_color = Color::LightCyan;
    let header_color = Color::Yellow;
    let label_color = Color::LightGray;
    let value_color = Color::White;
    let badge_color = if user.user_level.eq_ignore_ascii_case("sysop") {
        Color::LightMagenta
    } else {
        Color::LightGreen
    };
    let badge_text = if user.user_level.eq_ignore_ascii_case("sysop") {
        "[Sysop]"
    } else {
        "[User]"
    };

    // Inner width = 80 - 2 (border chars) = 78
    let inner = 78;

    // Helper: pad a string to a given width (left-aligned)
    let pad = |s: &str, width: usize| -> String {
        if s.len() >= width {
            s[..width].to_string()
        } else {
            format!("{}{}", s, " ".repeat(width - s.len()))
        }
    };

    // Top border
    w.set_fg(border_color);
    w.writeln(&format!("\u{2554}{}\u{2557}", "\u{2550}".repeat(inner)));

    // USER PROFILE header with badge
    let title = "USER PROFILE";
    // "  USER PROFILE" = 2 + 12 = 14 chars
    // badge_text at right side, with 2 spaces padding from right border
    // Total inner = 78
    // Left content: "  USER PROFILE" = 14
    // Right content: badge_text + "  " = badge_text.len() + 2
    // Middle spacing: 78 - 14 - badge_text.len() - 2
    let spacing = inner - 14 - badge_text.len() - 2;
    w.set_fg(border_color);
    w.write_str("\u{2551}  ");
    w.set_fg(header_color);
    w.bold();
    w.write_str(title);
    w.reset_color();
    w.write_str(&" ".repeat(spacing));
    w.set_fg(badge_color);
    w.bold();
    w.write_str(badge_text);
    w.reset_color();
    w.set_fg(border_color);
    w.writeln("  \u{2551}");

    // Separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(inner)));

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // User info fields
    let real_name = user.real_name.as_deref().unwrap_or("(not set)");
    let location = user.location.as_deref().unwrap_or("(not set)");
    let member_since = format_date(&user.created_at);
    let last_on = user
        .last_login
        .as_deref()
        .map(format_datetime)
        .unwrap_or_else(|| "Never".to_string());

    let fields: Vec<(&str, &str)> = vec![
        ("Handle:", &user.handle),
        ("Real Name:", real_name),
        ("Location:", location),
    ];

    for (label, value) in &fields {
        // "  Handle:    value" -- label is 12 chars padded, then value fills rest
        let label_padded = pad(label, 12);
        let value_str = *value;
        // line content: "  " + label(12) + value + padding to inner
        let content_len = 2 + 12 + value_str.len();
        let right_pad = if content_len < inner {
            inner - content_len
        } else {
            0
        };

        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(label_color);
        w.write_str(&label_padded);
        w.set_fg(value_color);
        w.bold();
        w.write_str(value_str);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Member since (using formatted date)
    {
        let label = "Member:";
        let label_padded = pad(label, 12);
        let value_str = &member_since;
        let content_len = 2 + 12 + value_str.len();
        let right_pad = if content_len < inner {
            inner - content_len
        } else {
            0
        };

        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(label_color);
        w.write_str(&label_padded);
        w.set_fg(value_color);
        w.bold();
        w.write_str(value_str);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Last On
    {
        let label = "Last On:";
        let label_padded = pad(label, 12);
        let value_str = &last_on;
        let content_len = 2 + 12 + value_str.len();
        let right_pad = if content_len < inner {
            inner - content_len
        } else {
            0
        };

        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(label_color);
        w.write_str(&label_padded);
        w.set_fg(value_color);
        w.bold();
        w.write_str(value_str);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // STATISTICS separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(inner)));

    // STATISTICS header
    {
        let title = "STATISTICS";
        let content_len = 2 + title.len();
        let right_pad = inner - content_len;
        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(header_color);
        w.bold();
        w.write_str(title);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(inner)));

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // Stats row 1: Total Calls + Messages Sent
    {
        let total_calls = format!("{}", user.total_logins);
        let messages_sent = format!("{}", user.messages_sent);

        // "  Total Calls:    47          Messages Sent:  23"
        // Left col: "  Total Calls:    " (18) + value
        // Right col starts around col 40
        let left_label = "Total Calls:";
        let right_label = "Messages Sent:";
        let left_val = pad(&total_calls, 12);
        let right_val = &messages_sent;

        // "  Total Calls:    47          Messages Sent:  23                            "
        // Build: "  " + left_label(14 padded) + "  " + left_val(12) + right_label(16 padded) + right_val + padding
        let left_part = format!("  {}  {}", pad(left_label, 14), left_val);
        let right_part = format!("{}  {}", pad(right_label, 14), right_val);
        let combined = format!("{}{}", left_part, right_part);
        let right_pad = if combined.len() < inner {
            inner - combined.len()
        } else {
            0
        };

        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(label_color);
        w.write_str(&pad(left_label, 14));
        w.write_str("  ");
        w.set_fg(value_color);
        w.bold();
        w.write_str(&left_val);
        w.reset_color();
        w.set_fg(label_color);
        w.write_str(&pad(right_label, 14));
        w.write_str("  ");
        w.set_fg(value_color);
        w.bold();
        w.write_str(right_val);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Stats row 2: Time Online + Games Played
    {
        let time_online = format_duration_minutes(user.total_time_minutes as i64);
        let games_played = format!("{}", user.games_played);

        let left_label = "Time Online:";
        let right_label = "Games Played:";
        let left_val = pad(&time_online, 12);
        let right_val = &games_played;

        let left_part = format!("  {}  {}", pad(left_label, 14), left_val);
        let right_part = format!("{}  {}", pad(right_label, 14), right_val);
        let combined = format!("{}{}", left_part, right_part);
        let right_pad = if combined.len() < inner {
            inner - combined.len()
        } else {
            0
        };

        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(label_color);
        w.write_str(&pad(left_label, 14));
        w.write_str("  ");
        w.set_fg(value_color);
        w.bold();
        w.write_str(&left_val);
        w.reset_color();
        w.set_fg(label_color);
        w.write_str(&pad(right_label, 14));
        w.write_str("  ");
        w.set_fg(value_color);
        w.bold();
        w.write_str(right_val);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Blank line
    w.set_fg(border_color);
    w.write_str("\u{2551}");
    w.write_str(&" ".repeat(inner));
    w.set_fg(border_color);
    w.writeln("\u{2551}");

    // SIGNATURE separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(inner)));

    // SIGNATURE header
    {
        let title = "SIGNATURE";
        let content_len = 2 + title.len();
        let right_pad = inner - content_len;
        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.set_fg(header_color);
        w.bold();
        w.write_str(title);
        w.reset_color();
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Separator
    w.set_fg(border_color);
    w.writeln(&format!("\u{2560}{}\u{2563}", "\u{2550}".repeat(inner)));

    // Signature content (up to 3 lines)
    let sig = user
        .signature
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("(none)");

    let sig_lines: Vec<&str> = sig.lines().take(3).collect();
    for line in &sig_lines {
        // Signature may contain ANSI codes, so we can't easily measure visible length.
        // For "(none)" and plain text, pad normally. For ANSI content, best-effort.
        let content_len = 2 + line.len();
        let right_pad = if content_len < inner {
            inner - content_len
        } else {
            0
        };
        w.set_fg(border_color);
        w.write_str("\u{2551}  ");
        w.reset_color();
        w.write_str(line);
        w.write_str(&" ".repeat(right_pad));
        w.set_fg(border_color);
        w.writeln("\u{2551}");
    }

    // Bottom border
    w.set_fg(border_color);
    w.writeln(&format!("\u{255A}{}\u{255D}", "\u{2550}".repeat(inner)));
    w.reset_color();

    w.flush()
}

/// Render the profile edit menu and return it as a string (for direct sending).
pub fn render_profile_edit_menu_string() -> String {
    let mut w = AnsiWriter::new();
    render_profile_edit_menu(&mut w);
    w.flush()
}

/// Render the profile edit menu as a single horizontal row.
pub fn render_profile_edit_menu(writer: &mut AnsiWriter) {
    writer.writeln("");
    writer.set_fg(Color::LightCyan);
    writer.write_str("  [1] ");
    writer.set_fg(Color::White);
    writer.write_str("Name  ");
    writer.set_fg(Color::LightCyan);
    writer.write_str("[2] ");
    writer.set_fg(Color::White);
    writer.write_str("Location  ");
    writer.set_fg(Color::LightCyan);
    writer.write_str("[3] ");
    writer.set_fg(Color::White);
    writer.write_str("Signature  ");
    writer.set_fg(Color::LightCyan);
    writer.write_str("[4] ");
    writer.set_fg(Color::White);
    writer.write_str("Bio  ");
    writer.set_fg(Color::LightCyan);
    writer.write_str("[Q] ");
    writer.set_fg(Color::White);
    writer.writeln("Back");
    writer.reset_color();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_duration_minutes(0), "0m");
    }

    #[test]
    fn test_format_duration_minutes_only() {
        assert_eq!(format_duration_minutes(45), "45m");
    }

    #[test]
    fn test_format_duration_hours_and_minutes() {
        assert_eq!(format_duration_minutes(90), "1h 30m");
    }

    #[test]
    fn test_format_duration_exact_hours() {
        assert_eq!(format_duration_minutes(1440), "24h 0m");
    }

    #[test]
    fn test_format_duration_one_hour() {
        assert_eq!(format_duration_minutes(60), "1h 0m");
    }

    #[test]
    fn test_format_date_standard() {
        assert_eq!(format_date("2026-01-26 14:15:00"), "January 26, 2026");
    }

    #[test]
    fn test_format_date_iso_t() {
        assert_eq!(format_date("2026-12-01T09:30:00"), "December 1, 2026");
    }

    #[test]
    fn test_format_datetime_with_time() {
        assert_eq!(
            format_datetime("2026-01-26 16:15:00"),
            "January 26, 2026 at 4:15 PM"
        );
    }

    #[test]
    fn test_format_datetime_midnight() {
        assert_eq!(
            format_datetime("2026-01-26 00:05:00"),
            "January 26, 2026 at 12:05 AM"
        );
    }

    #[test]
    fn test_format_datetime_noon() {
        assert_eq!(
            format_datetime("2026-01-26 12:00:00"),
            "January 26, 2026 at 12:00 PM"
        );
    }

    #[test]
    fn test_render_profile_card_contains_handle() {
        let user = make_test_user();
        let card = render_profile_card(&user, true);
        assert!(card.contains("TestUser"));
    }

    #[test]
    fn test_render_profile_card_contains_sections() {
        let user = make_test_user();
        let card = render_profile_card(&user, true);
        assert!(card.contains("USER PROFILE"));
        assert!(card.contains("STATISTICS"));
        assert!(card.contains("SIGNATURE"));
    }

    #[test]
    fn test_render_profile_card_optional_fields_none() {
        let mut user = make_test_user();
        user.real_name = None;
        user.location = None;
        user.signature = None;
        let card = render_profile_card(&user, true);
        assert!(card.contains("(not set)"));
        assert!(card.contains("(none)"));
    }

    #[test]
    fn test_render_profile_card_box_drawing() {
        let user = make_test_user();
        let card = render_profile_card(&user, true);
        assert!(card.contains("\u{2554}")); // top-left
        assert!(card.contains("\u{2557}")); // top-right
        assert!(card.contains("\u{255A}")); // bottom-left
        assert!(card.contains("\u{255D}")); // bottom-right
        assert!(card.contains("\u{2551}")); // vertical
        assert!(card.contains("\u{2550}")); // horizontal
    }

    fn make_test_user() -> User {
        User {
            id: 1,
            handle: "TestUser".to_string(),
            handle_lower: "testuser".to_string(),
            email: "test@example.com".to_string(),
            email_verified: 1,
            password_hash: "hash".to_string(),
            real_name: Some("Test Person".to_string()),
            location: Some("Newark, DE".to_string()),
            signature: Some("Stay curious.".to_string()),
            bio: Some("A test user".to_string()),
            user_level: "User".to_string(),
            created_at: "2026-01-26 14:00:00".to_string(),
            last_login: Some("2026-01-26 16:15:00".to_string()),
            total_logins: 47,
            total_time_minutes: 332,
            messages_sent: 23,
            games_played: 12,
            daily_time_used: 0,
            banked_time: 0,
            last_daily_reset: None,
        }
    }
}
