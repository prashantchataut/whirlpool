//! The "an update is available" reminder: a native Windows toast.
//!
//! This module only builds the PowerShell that shows the toast and decides it is safe to run;
//! starting it is the caller's job. Everything here is plain logic, so it is tested on any OS.
//! The text of the toast is built from a version string that has passed [`safe_version`], so no
//! release name from the network can inject script or markup.

/// The notification identity shown on the toast and in Windows' notification settings.
pub const APP_ID: &str = "Whirlpool";

/// True for strings that are only digits, letters, `.`, `-` and `+` (a version or a tag).
#[must_use]
pub fn safe_version(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 40
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
}

/// The PowerShell that registers the notification identity (per user, no admin rights) and shows
/// the toast. `None` when `version`, `running` or `tag` is not [`safe_version`].
#[must_use]
pub fn toast_script(version: &str, running: &str, tag: &str, repository: &str) -> Option<String> {
    if !(safe_version(version) && safe_version(running) && safe_version(tag)) {
        return None;
    }
    let url = format!("{repository}/releases/tag/{tag}");
    Some(format!(
        r#"$ErrorActionPreference = 'Stop'
$id = '{APP_ID}'
$key = "HKCU:\Software\Classes\AppUserModelId\$id"
if (-not (Test-Path $key)) {{ New-Item -Path $key -Force | Out-Null }}
New-ItemProperty -Path $key -Name DisplayName -Value 'Whirlpool' -PropertyType String -Force | Out-Null
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml('<toast activationType="protocol" launch="{url}"><visual><binding template="ToastGeneric"><text>Whirlpool {version} is available</text><text>You have {running}. Open Settings, then Updates, to install it. The download is checked against its SHA-256 first.</text></binding></visual><actions><action content="What is new" arguments="{url}" activationType="protocol"/><action content="Later" arguments="later" activationType="system"/></actions></toast>')
$toast = New-Object Windows.UI.Notifications.ToastNotification $xml
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($id).Show($toast)
"#
    ))
}

/// Text that is safe to put inside the toast's XML and PowerShell quotes: letters, digits, spaces and a
/// few marks, at most 90 characters.
#[must_use]
pub fn plain_text(text: &str) -> bool {
    !text.is_empty()
        && text.chars().count() <= 90
        && text
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '%' | '.' | ',' | '-' | ':'))
}

/// A plain notification with a title and one line, under Whirlpool's own identity. `None` when either
/// text is not [`plain_text`].
#[must_use]
pub fn message_toast_script(title: &str, body: &str) -> Option<String> {
    if !(plain_text(title) && plain_text(body)) {
        return None;
    }
    Some(format!(
        r#"$ErrorActionPreference = 'Stop'
$id = '{APP_ID}'
$key = "HKCU:\Software\Classes\AppUserModelId\$id"
if (-not (Test-Path $key)) {{ New-Item -Path $key -Force | Out-Null }}
New-ItemProperty -Path $key -Name DisplayName -Value 'Whirlpool' -PropertyType String -Force | Out-Null
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml('<toast><visual><binding template="ToastGeneric"><text>{title}</text><text>{body}</text></binding></visual></toast>')
$toast = New-Object Windows.UI.Notifications.ToastNotification $xml
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($id).Show($toast)
"#
    ))
}

/// `powershell -EncodedCommand` wants the script as base64 of UTF-16LE.
#[must_use]
pub fn encoded_command(script: &str) -> String {
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    base64(&bytes)
}

/// Standard base64 with padding.
#[must_use]
pub fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(char::from(TABLE[(n >> 18) as usize & 63]));
        out.push(char::from(TABLE[(n >> 12) as usize & 63]));
        out.push(if chunk.len() > 1 {
            char::from(TABLE[(n >> 6) as usize & 63])
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            char::from(TABLE[n as usize & 63])
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_rfc_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn encoded_command_is_utf16le() {
        // "A" is 0x41 0x00 in UTF-16LE.
        assert_eq!(encoded_command("A"), "QQA=");
    }

    #[test]
    fn only_plain_versions_get_a_toast() {
        assert!(safe_version("4.3.0-rc1+build.5"));
        assert!(!safe_version(""));
        assert!(!safe_version("4.3.0'; calc; '"));
        assert!(!safe_version("<b>"));
        assert!(!safe_version(&"1".repeat(41)));
        let repo = "https://github.com/x/y";
        assert!(toast_script("4.3.0", "4.2.0", "v4.3.0", repo).is_some());
        assert!(toast_script("4.3.0'", "4.2.0", "v4.3.0", repo).is_none());
        assert!(toast_script("4.3.0", "4.2.0", "v4.3.0\"", repo).is_none());
    }

    #[test]
    fn the_script_names_both_versions_and_the_release_page() {
        let script = toast_script("4.3.0", "4.2.0", "v4.3.0", "https://github.com/x/y").unwrap();
        assert!(script.contains("Whirlpool 4.3.0 is available"));
        assert!(script.contains("You have 4.2.0"));
        assert!(script.contains("https://github.com/x/y/releases/tag/v4.3.0"));
        assert!(script.contains("AppUserModelId"));
    }

    #[test]
    fn message_toast_refuses_text_that_could_break_out() {
        assert!(message_toast_script("Battery at 20%", "Plug in the charger.").is_some());
        assert!(message_toast_script("it's", "x").is_none());
        assert!(message_toast_script("a", "<b>x</b>").is_none());
        assert!(message_toast_script("a", "$(calc)").is_none());
        assert!(message_toast_script("", "x").is_none());
        assert!(message_toast_script(&"a".repeat(91), "x").is_none());
    }
}
