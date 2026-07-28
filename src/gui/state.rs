use std::path::PathBuf;

/// クリップのメディアタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MediaType {
    Video,
    Audio,
    Text,
    Effect,
}

impl MediaType {
    pub fn label(&self) -> &'static str {
        match self {
            MediaType::Video => "映像",
            MediaType::Audio => "音声",
            MediaType::Text => "テキスト",
            MediaType::Effect => "エフェクト",
        }
    }

    pub fn default_color(&self) -> u32 {
        match self {
            MediaType::Video => 0x4682B4,   // 青
            MediaType::Audio => 0x40C040,   // 緑
            MediaType::Text => 0xDAA520,    // 金
            MediaType::Effect => 0xCD5C5C,  // 赤
        }
    }
}

/// タイムライン上の1つのトラック
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Track {
    pub name: String,
    pub index: usize,
    pub clips: Vec<ClipData>,
    pub visible: bool,
    pub locked: bool,
}

impl Track {
    pub fn new(name: &str, index: usize) -> Self {
        Self {
            name: name.to_string(),
            index,
            clips: Vec::new(),
            visible: true,
            locked: false,
        }
    }
}

/// クリップデータ（プロジェクト保存用 + 実行時）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipData {
    /// ファイルパス（動画/音声ファイルの場合）
    pub file_path: Option<String>,
    /// クリップの表示ラベル
    pub label: String,
    /// タイムライン上の開始位置（秒）
    pub start_time: f64,
    /// クリップの長さ（秒）
    pub duration: f64,
    /// ソースメディアのIn点（秒）
    pub in_point: f64,
    /// ソースメディアのOut点（秒）
    pub out_point: f64,
    /// メディアタイプ
    pub media_type: MediaType,
    /// 所属トラックのインデックス
    pub track_idx: usize,
    /// クリップの一意なID
    pub id: u64,
}

/// プロジェクト全体のデータ構造
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub tracks: Vec<Track>,
    pub duration: f64,
    pub settings: ProjectSettings,
    /// セリフテキスト（プロジェクト保存用）
    pub script_text: String,
    next_clip_id: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectSettings {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub sample_rate: u32,
}

impl Default for Project {
    fn default() -> Self {
        let mut project = Self {
            name: "新規プロジェクト".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            tracks: Vec::new(),
            duration: 0.0,
            settings: ProjectSettings {
                width: 1280,
                height: 720,
                fps: 30.0,
                sample_rate: 44100,
            },
            script_text: String::new(),
            next_clip_id: 0,
        };
        // デフォルトで1つの汎用トラックを作成
        project.add_track("トラック 1");
        project
    }
}

impl Project {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// トラックを追加
    pub fn add_track(&mut self, name: &str) -> usize {
        let idx = self.tracks.len();
        self.tracks.push(Track::new(name, idx));
        idx
    }

    /// トラックを削除
    pub fn remove_track(&mut self, index: usize) {
        self.tracks.retain(|t| t.index != index);
        // インデックスを振り直し
        for (i, track) in self.tracks.iter_mut().enumerate() {
            track.index = i;
        }
    }

    /// クリップを追加
    pub fn add_clip(
        &mut self,
        file_path: Option<String>,
        label: &str,
        start_time: f64,
        duration: f64,
        media_type: MediaType,
        track_idx: usize,
    ) -> u64 {
        let id = self.next_clip_id;
        self.next_clip_id += 1;

        let clip = ClipData {
            file_path,
            label: label.to_string(),
            start_time,
            duration,
            in_point: 0.0,
            out_point: duration,
            media_type,
            track_idx,
            id,
        };

        // トラックを見つけて追加
        if let Some(track) = self.tracks.get_mut(track_idx) {
            track.clips.push(clip);
        }

        // 全体の長さを更新
        self.update_duration();

        id
    }

    /// クリップを削除
    pub fn remove_clip(&mut self, clip_id: u64) -> Option<ClipData> {
        for track in &mut self.tracks {
            if let Some(pos) = track.clips.iter().position(|c| c.id == clip_id) {
                let clip = track.clips.remove(pos);
                self.update_duration();
                return Some(clip);
            }
        }
        None
    }

    /// クリップをIDで検索
    pub fn find_clip(&self, clip_id: u64) -> Option<&ClipData> {
        for track in &self.tracks {
            if let Some(clip) = track.clips.iter().find(|c| c.id == clip_id) {
                return Some(clip);
            }
        }
        None
    }

    /// クリップをIDで検索（可変参照）
    pub fn find_clip_mut(&mut self, clip_id: u64) -> Option<&mut ClipData> {
        for track in &mut self.tracks {
            if let Some(clip) = track.clips.iter_mut().find(|c| c.id == clip_id) {
                return Some(clip);
            }
        }
        None
    }

    /// クリップを別のトラックへ移動
    pub fn move_clip_to_track(&mut self, clip_id: u64, new_track_idx: usize) -> bool {
        // まずクリップを探して取り出す
        let clip_opt = self.remove_clip(clip_id);
        if let Some(clip) = clip_opt {
            if let Some(track) = self.tracks.get_mut(new_track_idx) {
                track.clips.push(clip);
                return true;
            }
        }
        false
    }

    /// 全体の長さを再計算
    pub fn update_duration(&mut self) {
        self.duration = self
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .map(|c| c.start_time + c.duration)
            .fold(0.0f64, f64::max);
    }

    /// すべてのクリップを収集（タイムライン順）
    pub fn all_clips(&self) -> Vec<&ClipData> {
        let mut clips: Vec<&ClipData> = self.tracks.iter().flat_map(|t| t.clips.iter()).collect();
        clips.sort_by(|a, b| {
            a.start_time
                .partial_cmp(&b.start_time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        clips
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

/// ============================================================
/// アプリケーション全体の状態（実行時用）
/// ============================================================

/// アプリケーションの再生状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// アプリケーション全体の状態
pub struct AppState {
    /// 現在のプロジェクト
    pub project: Project,
    /// アンドゥ/リドゥスタック
    pub undo_stack: UndoStack,
    /// 再生状態
    pub playback_state: PlaybackState,
    /// 現在の再生位置（秒）
    pub playback_position: f64,
    /// 選択中のクリップID
    pub selected_clip_id: Option<u64>,
    /// 選択中のトラックインデックス
    pub selected_track_idx: Option<usize>,
    /// 現在のズームレベル（pixels per second）
    pub zoom_level: f64,
    /// タイムラインの水平スクロール位置（秒）
    pub scroll_offset: f64,
    /// プロジェクトファイルの保存先パス
    pub project_file_path: Option<PathBuf>,
    /// 変更フラグ（未保存の変更あり）
    pub is_modified: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            project: Project::default(),
            undo_stack: UndoStack::new(100),
            playback_state: PlaybackState::Stopped,
            playback_position: 0.0,
            selected_clip_id: None,
            selected_track_idx: None,
            zoom_level: 100.0,
            scroll_offset: 0.0,
            project_file_path: None,
            is_modified: false,
        }
    }

    /// 再生位置をセット
    pub fn set_playback_position(&mut self, pos: f64) {
        self.playback_position = pos.max(0.0).min(self.project.duration);
    }

    /// 選択クリップを取得
    pub fn selected_clip(&self) -> Option<&ClipData> {
        self.selected_clip_id
            .and_then(|id| self.project.find_clip(id))
    }
}

/// ============================================================
/// アンドゥ/リドゥ用コマンドパターン
/// ============================================================

#[derive(Debug, Clone)]
pub enum EditCommand {
    AddClip {
        clip: ClipData,
    },
    RemoveClip {
        clip: ClipData,
    },
    MoveClip {
        clip_id: u64,
        old_start_time: f64,
        new_start_time: f64,
        old_track_idx: usize,
        new_track_idx: usize,
    },
    TrimClip {
        clip_id: u64,
        old_in_point: f64,
        old_out_point: f64,
        old_duration: f64,
        new_in_point: f64,
        new_out_point: f64,
        new_duration: f64,
    },
    SplitClip {
        original_clip: ClipData,
        new_clip: ClipData,
    },
}

/// アンドゥスタック
pub struct UndoStack {
    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
    max_depth: usize,
}

impl UndoStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    pub fn push(&mut self, cmd: EditCommand) {
        self.undo_stack.push(cmd);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<EditCommand> {
        let cmd = self.undo_stack.pop()?;
        let result = cmd.clone();
        self.redo_stack.push(cmd);
        Some(result)
    }

    pub fn redo(&mut self) -> Option<EditCommand> {
        let cmd = self.redo_stack.pop()?;
        let result = cmd.clone();
        self.undo_stack.push(cmd);
        Some(result)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}