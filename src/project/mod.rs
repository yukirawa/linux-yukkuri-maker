use serde::{Deserialize, Serialize};

/// プロジェクト全体のデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub timeline: TimelineData,
    pub settings: ProjectSettings,
}

/// タイムラインデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineData {
    pub clips: Vec<ClipData>,
    pub duration: f64,
}

/// クリップデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipData {
    pub file_path: String,
    pub start_time: f64,
    pub end_time: f64,
    pub track_index: usize,
}

/// プロジェクト設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "新規プロジェクト".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timeline: TimelineData {
                clips: Vec::new(),
                duration: 0.0,
            },
            settings: ProjectSettings {
                width: 1280,
                height: 720,
                fps: 30.0,
            },
        }
    }
}

impl Project {
    /// 新しいプロジェクトを作成
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// JSONとしてシリアライズ
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// JSONからデシリアライズ
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// ファイルに保存
    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// ファイルから読み込み
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }
}