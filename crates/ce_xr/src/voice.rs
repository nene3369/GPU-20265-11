/// Voice input configuration.
pub struct VoiceConfig {
    /// Language for speech recognition (BCP47 code).
    pub language: String,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Enable wake word detection.
    pub wake_word: Option<String>,
    /// Voice activity detection threshold (0.0 - 1.0).
    pub vad_threshold: f32,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            language: "ja-JP".to_string(),
            sample_rate: 16000,
            wake_word: None,
            vad_threshold: 0.5,
        }
    }
}

/// Recognized speech event.
#[derive(Debug, Clone)]
pub struct SpeechEvent {
    /// Transcribed text.
    pub text: String,
    /// Confidence of recognition (0.0 - 1.0).
    pub confidence: f32,
    /// Language detected (BCP47 code).
    pub language: String,
    /// Duration of the utterance in seconds.
    pub duration_seconds: f32,
    /// Whether the speech is still ongoing (partial result).
    pub is_partial: bool,
}

/// Voice activity detection state.
#[derive(Debug, Clone, Copy, Default)]
pub struct VoiceActivity {
    pub is_speaking: bool,
    /// Root-mean-square volume level.
    pub volume_rms: f32,
    /// Duration of current speech/silence segment in seconds.
    pub duration_seconds: f32,
}

/// Voice input resource, inserted when voice recognition is enabled.
#[derive(Debug, Clone, Default)]
pub struct VoiceInput {
    pub activity: VoiceActivity,
    pub last_speech: Option<SpeechEvent>,
    pub is_listening: bool,
    pub provider: VoiceProvider,
}

/// Identifies the speech recognition backend in use.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VoiceProvider {
    #[default]
    None,
    /// whisper.cpp / whisper-rs (local inference).
    WhisperLocal,
    /// OS speech-to-text API.
    SystemSTT,
    /// Cloud-based (Google, Azure, etc.).
    CloudAPI,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_config_default_is_japanese() {
        let cfg = VoiceConfig::default();
        assert_eq!(cfg.language, "ja-JP");
        assert_eq!(cfg.sample_rate, 16000);
        assert!(cfg.wake_word.is_none());
    }

    #[test]
    fn voice_input_default_not_listening() {
        let vi = VoiceInput::default();
        assert!(!vi.is_listening);
        assert!(vi.last_speech.is_none());
        assert_eq!(vi.provider, VoiceProvider::None);
    }

    #[test]
    fn voice_activity_default_not_speaking() {
        let va = VoiceActivity::default();
        assert!(!va.is_speaking);
        assert_eq!(va.volume_rms, 0.0);
    }
}
