//! Where a notification opens. Android's tap and a desktop click share these
//! route names; the WebView resolves them (`checkin` is the latest review, and
//! `sleep` leads with Tonight from the evening on).

/// Every route a notification may open. Android's `VaultPlugin` allows these
/// too; a click carrying anything else opens nothing.
#[cfg(any(desktop, test))]
pub const TARGETS: [&str; 5] = ["today", "checkin", "health", "training", "sleep"];

pub fn notification_target(kind: &str) -> &'static str {
    match kind {
        "morning" => "today",
        "weekly" => "checkin",
        "vitals" | "fitness-age" | "vo2max" => "health",
        "training-status" => "training",
        "bedtime" => "sleep",
        _ => "today",
    }
}

/// The route a desktop click opens: only a click on the notification itself
/// (`tap`), never one of its actions, and only a known target.
#[cfg(desktop)]
pub fn clicked(action_id: &str, target: Option<&serde_json::Value>) -> Option<&'static str> {
    let target = target?.as_str()?;
    (action_id == "tap")
        .then(|| TARGETS.into_iter().find(|known| *known == target))
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_notification_opens_a_known_target() {
        let kinds = ["morning", "weekly", "vitals", "fitness-age", "vo2max", "training-status", "bedtime"];
        let targets: Vec<_> = kinds.iter().map(|k| notification_target(k)).collect();
        assert_eq!(targets, ["today", "checkin", "health", "health", "health", "training", "sleep"]);
        assert!(targets.iter().all(|t| TARGETS.contains(t)));
        assert_eq!(notification_target("unknown"), "today");
    }

    #[cfg(desktop)]
    #[test]
    fn only_a_click_on_a_notification_with_a_known_target_opens_it() {
        use serde_json::json;
        assert_eq!(clicked("tap", Some(&json!("checkin"))), Some("checkin"));
        assert_eq!(clicked("tap", Some(&json!("sleep"))), Some("sleep"));
        assert_eq!(clicked("snooze", Some(&json!("today"))), None);
        assert_eq!(clicked("tap", Some(&json!("settings"))), None);
        assert_eq!(clicked("tap", Some(&json!("#today"))), None);
        assert_eq!(clicked("tap", Some(&json!(1))), None);
        assert_eq!(clicked("tap", None), None);
    }
}
