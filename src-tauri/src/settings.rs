//! App-owned preferences. They shape display and reminders, never facts.
use serde::{Deserialize, Serialize};
use temporal_core::time::ZoneId;

pub const FALLBACK_ZONE: &str = "America/Toronto";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeekStart {
    #[default]
    Mon,
    Sun,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    /// IANA zone used to display time and bound the daily edit. Unset until the
    /// app first learns the system zone; dated records keep their own zones.
    pub display_zone: Option<String>,
    pub week_starts_on: WeekStart,
    /// Minutes before a fixed event to show a reminder; `None` turns them off.
    pub reminder_minutes: Option<u32>,
    pub theme: Theme,
    pub default_event_minutes: u32,
    /// Keep running in the notification area when the window closes, so
    /// reminders continue.
    pub keep_running_in_tray: bool,
    /// First hour shown when a day or week opens.
    pub day_start_hour: u8,
    /// The Trace export file last chosen, re-read when it changes.
    pub trace_export_path: Option<String>,
    /// Start quietly in the notification area at sign-in (desktop only).
    pub open_at_login: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display_zone: None,
            week_starts_on: WeekStart::Mon,
            reminder_minutes: Some(10),
            theme: Theme::System,
            default_event_minutes: 60,
            keep_running_in_tray: true,
            day_start_hour: 8,
            trace_export_path: None,
            open_at_login: false,
        }
    }
}

/// A partial update from the Settings page. `None` leaves a field unchanged;
/// the explicit `clear_*` flags remove optional values.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SettingsPatch {
    pub display_zone: Option<String>,
    pub week_starts_on: Option<WeekStart>,
    pub reminder_minutes: Option<u32>,
    pub reminders_off: bool,
    pub theme: Option<Theme>,
    pub default_event_minutes: Option<u32>,
    pub keep_running_in_tray: Option<bool>,
    pub day_start_hour: Option<u8>,
    pub open_at_login: Option<bool>,
}

impl Settings {
    pub fn zone(&self) -> ZoneId {
        self.display_zone
            .as_deref()
            .and_then(|zone| zone.parse().ok())
            .unwrap_or_else(|| FALLBACK_ZONE.parse().expect("bundled fallback zone"))
    }

    pub fn apply(&mut self, patch: &SettingsPatch) -> Result<(), String> {
        if let Some(zone) = &patch.display_zone {
            zone.parse::<ZoneId>()
                .map_err(|_| format!("{zone} is not a known IANA time zone."))?;
            self.display_zone = Some(zone.clone());
        }
        if let Some(value) = patch.week_starts_on {
            self.week_starts_on = value;
        }
        if patch.reminders_off {
            self.reminder_minutes = None;
        } else if let Some(minutes) = patch.reminder_minutes {
            if minutes > 24 * 60 {
                return Err("Reminders can be at most one day early.".into());
            }
            self.reminder_minutes = Some(minutes);
        }
        if let Some(theme) = patch.theme {
            self.theme = theme;
        }
        if let Some(minutes) = patch.default_event_minutes {
            if !(5..=24 * 60).contains(&minutes) {
                return Err("A new event lasts between 5 minutes and one day.".into());
            }
            self.default_event_minutes = minutes;
        }
        if let Some(value) = patch.keep_running_in_tray {
            self.keep_running_in_tray = value;
        }
        if let Some(value) = patch.open_at_login {
            self.open_at_login = value;
        }
        if let Some(hour) = patch.day_start_hour {
            if hour > 23 {
                return Err("Choose an hour from 0 to 23.".into());
            }
            self.day_start_hour = hour;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_validated_patches() {
        let mut settings = Settings::default();
        assert_eq!(settings.zone().name(), FALLBACK_ZONE);
        settings
            .apply(&SettingsPatch {
                display_zone: Some("Europe/Paris".into()),
                reminders_off: true,
                day_start_hour: Some(6),
                ..SettingsPatch::default()
            })
            .unwrap();
        assert_eq!(settings.zone().name(), "Europe/Paris");
        assert_eq!(settings.reminder_minutes, None);
        assert!(
            settings
                .apply(&SettingsPatch {
                    display_zone: Some("Mars/Base".into()),
                    ..SettingsPatch::default()
                })
                .is_err()
        );
        assert_eq!(settings.zone().name(), "Europe/Paris");
        let legacy: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(legacy, Settings::default());
    }
}
