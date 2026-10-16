//! Garmin's single sign-on, as used from a webview.
//!
//! The app shows Garmin's own sign-in page (MFA included). After sign-in the
//! page redirects to the service URL with a one-time CAS ticket
//! (`?ticket=ST-...`); the app intercepts that navigation and exchanges the
//! ticket for OAuth tokens. The ticket must be exchanged with the same
//! service URL it was issued for.

/// Garmin Connect's web portal client.
pub const PORTAL_CLIENT_ID: &str = "GarminConnect";

/// Where Garmin redirects after sign-in, carrying the ticket.
pub const PORTAL_SERVICE_URL: &str = "https://connect.garmin.com/app";

/// Garmin's sign-in page for the portal client.
pub fn sign_in_url() -> String {
    format!(
        "https://sso.garmin.com/portal/sso/en-US/sign-in?clientId={PORTAL_CLIENT_ID}&service={}",
        encode(PORTAL_SERVICE_URL)
    )
}

/// The ticket from a navigation to the service URL, if this is that
/// navigation. Any other URL returns `None`.
pub fn extract_ticket(url: &str) -> Option<String> {
    let rest = url.strip_prefix(PORTAL_SERVICE_URL)?;
    let query = rest.strip_prefix('/').unwrap_or(rest).strip_prefix('?')?;
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == "ticket")
        .map(|(_, value)| value.to_owned())
        .filter(|ticket| ticket.starts_with("ST-"))
}

fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_in_url_encodes_the_service() {
        assert_eq!(
            sign_in_url(),
            "https://sso.garmin.com/portal/sso/en-US/sign-in?clientId=GarminConnect&service=https%3A%2F%2Fconnect.garmin.com%2Fapp"
        );
    }

    #[test]
    fn extracts_the_ticket_from_the_service_redirect() {
        assert_eq!(
            extract_ticket("https://connect.garmin.com/app?ticket=ST-0123-abc-cas"),
            Some("ST-0123-abc-cas".into())
        );
        assert_eq!(
            extract_ticket("https://connect.garmin.com/app/?foo=1&ticket=ST-9"),
            Some("ST-9".into())
        );
    }

    #[test]
    fn ignores_other_urls_and_non_tickets() {
        assert_eq!(
            extract_ticket("https://sso.garmin.com/portal/sso/en-US/sign-in"),
            None
        );
        assert_eq!(extract_ticket("https://connect.garmin.com/app"), None);
        assert_eq!(
            extract_ticket("https://connect.garmin.com/app?ticket=XX-1"),
            None
        );
        assert_eq!(
            extract_ticket("https://connect.garmin.com/apps?ticket=ST-1"),
            None
        );
        assert_eq!(
            extract_ticket("https://evil.example/?u=https://connect.garmin.com/app?ticket=ST-1"),
            None
        );
    }
}
