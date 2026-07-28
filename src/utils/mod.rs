// ユーティリティモジュール

/// 時間フォーマット関連のユーティリティ
pub mod time {
    /// 秒数を "M:SS" または "H:MM:SS" 形式にフォーマット
    pub fn format_duration(secs: f64) -> String {
        let total_secs = secs.round() as i64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        }
    }

    /// "M:SS" または "H:MM:SS" 形式の文字列を秒数にパース
    pub fn parse_duration(text: &str) -> Option<f64> {
        let parts: Vec<&str> = text.split(':').collect();
        match parts.len() {
            2 => {
                let mins: f64 = parts[0].parse().ok()?;
                let secs: f64 = parts[1].parse().ok()?;
                Some(mins * 60.0 + secs)
            }
            3 => {
                let hours: f64 = parts[0].parse().ok()?;
                let mins: f64 = parts[1].parse().ok()?;
                let secs: f64 = parts[2].parse().ok()?;
                Some(hours * 3600.0 + mins * 60.0 + secs)
            }
            _ => None,
        }
    }

    /// フレーム番号をタイムコードに変換
    pub fn frame_to_timecode(frame: u64, fps: f64) -> String {
        let total_secs = frame as f64 / fps;
        format_duration(total_secs)
    }
}

/// ファイルパス関連のユーティリティ
pub mod file {
    /// ファイルパスからファイル名（拡張子なし）を抽出
    pub fn stem(path: &str) -> Option<String> {
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    /// ファイルパスから拡張子（小文字）を抽出
    pub fn extension_lower(path: &str) -> Option<String> {
        std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
    }

    /// 動画ファイルかどうかを判定
    pub fn is_video_file(path: &str) -> bool {
        matches!(
            extension_lower(path).as_deref(),
            Some("mp4" | "mov" | "avi" | "mkv" | "webm" | "wmv" | "flv" | "ts" | "m4v" | "mpg" | "mpeg")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(time::format_duration(0.0), "0:00");
        assert_eq!(time::format_duration(65.0), "1:05");
        assert_eq!(time::format_duration(3661.0), "1:01:01");
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(time::parse_duration("1:05"), Some(65.0));
        assert_eq!(time::parse_duration("0:00"), Some(0.0));
        assert_eq!(time::parse_duration("1:01:01"), Some(3661.0));
        assert_eq!(time::parse_duration("abc"), None);
    }
}