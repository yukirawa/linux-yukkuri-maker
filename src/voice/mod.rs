// ボイス処理モジュール
// 第2弾で実装予定：
// - AquesTalk による音声合成
// - VOICEVOX HTTP API による音声合成
// - 音声ファイルの読み込み

/// ボイスエンジンの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceEngine {
    AquesTalk,
    VoiceVox,
}

/// ボイスパラメータ
#[derive(Debug, Clone)]
pub struct VoiceParams {
    /// 声の高さ
    pub pitch: f64,
    /// 話速
    pub speed: f64,
    /// 音量
    pub volume: f64,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            pitch: 1.0,
            speed: 1.0,
            volume: 1.0,
        }
    }
}

/// ボイス生成結果
#[derive(Debug, Clone)]
pub struct VoiceResult {
    /// WAV形式の音声データ
    pub wav_data: Vec<u8>,
    /// サンプルレート
    pub sample_rate: u32,
    /// 音声の長さ（秒）
    pub duration: f64,
}