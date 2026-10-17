// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Modular, downloadable animation templates for video, image, and text clips.
//!
//! An animation template declares how a clip's transform and visibility
//! properties (scale, offset, rotation, opacity) evolve across a normalised
//! time window [0.0, 1.0]. Templates are serialised as compact JSON files,
//! allowing offline built-in presets and dynamic online catalog downloads
//! without recompiling the application.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Command;
use crate::model::ranges::{MAX_OFFSET, MAX_SCALE, MIN_SCALE};
use crate::model::{Clip, KeyEase, KeyProperty};

/// The phase of a clip that the animation targets.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimationKind {
    /// Plays at the start of a clip (0.0 to t_in).
    In,
    /// Plays at the end of a clip (1.0 - t_out to 1.0).
    Out,
    /// Plays throughout the clip duration.
    Combo,
}

/// The clip types an animation template can be applied to.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimationTarget {
    /// Video clips.
    Video,
    /// Static picture clips.
    Image,
    /// Text titles and subtitles.
    Text,
    /// Geometric shape clips.
    Shape,
    /// Compatible with any visual clip.
    All,
}

/// How a keyframe value is calculated relative to the clip's resting state.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum AnimValue {
    /// Multiplied by the clip's resting value (e.g. resting_scale * 1.2).
    Relative(f64),
    /// Added to the clip's resting value (e.g. resting_rotation + (-360.0)).
    Offset(f64),
    /// An absolute, fixed value (e.g. opacity 0.0 or 1.0).
    Absolute(f64),
}

impl AnimValue {
    /// Computes the final numeric value given the clip's resting baseline.
    pub fn resolve(self, resting: f64) -> f64 {
        match self {
            AnimValue::Relative(factor) => resting * factor,
            AnimValue::Offset(delta) => resting + delta,
            AnimValue::Absolute(val) => val,
        }
    }
}

/// One keyframe in a normalised animation timeline [0.0, 1.0].
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AnimKey {
    /// Normalised time within the animation window [0.0, 1.0].
    pub t: f64,
    /// The target value calculation.
    pub value: AnimValue,
    /// Cubic bezier easing curve leading up to this keyframe.
    #[serde(default)]
    pub ease: KeyEase,
}

/// A complete, serialisable animation template.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AnimationTemplate {
    /// Unique identifier for this template (e.g. "zoom_in_bounce").
    pub id: String,
    /// Display name shown in UI panels (e.g. "Zoom In Bounce").
    pub name: String,
    /// Whether this animation is for In, Out, or Combo.
    pub kind: AnimationKind,
    /// Supported clip targets.
    #[serde(default = "default_targets")]
    pub targets: Vec<AnimationTarget>,
    /// Recommended default duration in seconds.
    #[serde(default = "default_duration")]
    pub default_duration: f64,
    /// Minimum allowed duration in seconds.
    #[serde(default = "default_min_duration")]
    pub min_duration: f64,
    /// Maximum allowed duration in seconds.
    #[serde(default = "default_max_duration")]
    pub max_duration: f64,
    /// Animated tracks mapped by property name.
    pub tracks: HashMap<KeyProperty, Vec<AnimKey>>,
}

fn default_targets() -> Vec<AnimationTarget> {
    vec![AnimationTarget::All]
}

fn default_duration() -> f64 {
    0.5
}

fn default_min_duration() -> f64 {
    0.1
}

fn default_max_duration() -> f64 {
    3.0
}

impl AnimationTemplate {
    /// Parses an animation template from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Serialises the animation template to a formatted JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Loads an animation template from a file on disk.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Generates the `SetClipKey` commands required to apply this template to
    /// `clip`.
    ///
    /// The generated keyframe points are mapped into the clip's normalised
    /// timeline `[0.0, 1.0]`, resting values are preserved, and properties
    /// are clamped within safe bounds.
    pub fn generate_commands(
        &self,
        clip: &Clip,
        clip_duration: f64,
        anim_duration: f64,
    ) -> Vec<Command> {
        let clip_dur = clip_duration.max(0.1);
        let safe_anim_dur = anim_duration.min(clip_dur * 0.48).max(0.05);
        let window_ratio = (safe_anim_dur / clip_dur).clamp(0.02, 0.49);

        let resting_scale = clip
            .value_at(KeyProperty::Scale, 0.5)
            .clamp(MIN_SCALE, MAX_SCALE);
        let resting_x = clip
            .value_at(KeyProperty::OffsetX, 0.5)
            .clamp(-MAX_OFFSET, MAX_OFFSET);
        let resting_y = clip
            .value_at(KeyProperty::OffsetY, 0.5)
            .clamp(-MAX_OFFSET, MAX_OFFSET);
        let resting_rot = clip.value_at(KeyProperty::Rotation, 0.5);
        let resting_opacity = clip.value_at(KeyProperty::Opacity, 0.5).clamp(0.0, 1.0);

        let mut commands = Vec::new();

        for (&prop, keys) in &self.tracks {
            let resting = match prop {
                KeyProperty::Scale => resting_scale,
                KeyProperty::OffsetX => resting_x,
                KeyProperty::OffsetY => resting_y,
                KeyProperty::Rotation => resting_rot,
                KeyProperty::Opacity => resting_opacity,
                KeyProperty::Volume => 1.0,
            };

            for key in keys {
                let norm_t = key.t.clamp(0.0, 1.0);
                let clip_point = match self.kind {
                    AnimationKind::In => norm_t * window_ratio,
                    AnimationKind::Out => (1.0 - window_ratio) + (norm_t * window_ratio),
                    AnimationKind::Combo => norm_t,
                };

                let raw_val = key.value.resolve(resting);
                let clamped_val = match prop {
                    KeyProperty::Scale => raw_val.clamp(MIN_SCALE, MAX_SCALE),
                    KeyProperty::OffsetX | KeyProperty::OffsetY => {
                        raw_val.clamp(-MAX_OFFSET, MAX_OFFSET)
                    }
                    KeyProperty::Opacity => raw_val.clamp(0.0, 1.0),
                    KeyProperty::Rotation | KeyProperty::Volume => raw_val,
                };

                commands.push(Command::SetClipKey {
                    clip_id: clip.id.clone(),
                    property: prop,
                    at: clip_point.clamp(0.0, 1.0),
                    value: clamped_val,
                    ease: key.ease,
                });
            }
        }

        commands
    }
}

/// A collection of animation templates, indexed by ID and category.
#[derive(Default, Clone, Debug)]
pub struct AnimationCatalog {
    templates: HashMap<String, AnimationTemplate>,
}

impl AnimationCatalog {
    /// Creates a new, empty catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a template in the catalog.
    pub fn insert(&mut self, template: AnimationTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Retrieves a template by its unique ID.
    pub fn get(&self, id: &str) -> Option<&AnimationTemplate> {
        self.templates.get(id)
    }

    /// Lists all templates belonging to a specific kind (`In`, `Out`, or
    /// `Combo`).
    pub fn list_by_kind(&self, kind: AnimationKind) -> Vec<&AnimationTemplate> {
        let mut list: Vec<_> = self
            .templates
            .values()
            .filter(|tmpl| tmpl.kind == kind)
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    /// Loads all `.json` animation files located in `directory`.
    pub fn load_directory(&mut self, directory: impl AsRef<Path>) -> usize {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return 0;
        };

        let mut loaded = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if let Ok(template) = AnimationTemplate::from_file(&path) {
                    self.insert(template);
                    loaded += 1;
                }
            }
        }
        loaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_json_roundtrip() {
        let mut tracks = HashMap::new();
        tracks.insert(
            KeyProperty::Scale,
            vec![
                AnimKey {
                    t: 0.0,
                    value: AnimValue::Relative(0.05),
                    ease: KeyEase::OUT,
                },
                AnimKey {
                    t: 1.0,
                    value: AnimValue::Relative(1.0),
                    ease: KeyEase::LINEAR,
                },
            ],
        );

        let template = AnimationTemplate {
            id: "zoom_in".to_string(),
            name: "Zoom In".to_string(),
            kind: AnimationKind::In,
            targets: vec![AnimationTarget::All],
            default_duration: 0.5,
            min_duration: 0.1,
            max_duration: 3.0,
            tracks,
        };

        let json = template.to_json().expect("serialisation failed");
        let parsed = AnimationTemplate::from_json(&json).expect("deserialisation failed");
        assert_eq!(template.id, parsed.id);
        assert_eq!(template.name, parsed.name);
        assert_eq!(template.kind, parsed.kind);
    }

    #[test]
    fn test_generate_commands() {
        let mut tracks = HashMap::new();
        tracks.insert(
            KeyProperty::Scale,
            vec![
                AnimKey {
                    t: 0.0,
                    value: AnimValue::Relative(0.05),
                    ease: KeyEase::OUT,
                },
                AnimKey {
                    t: 1.0,
                    value: AnimValue::Relative(1.0),
                    ease: KeyEase::LINEAR,
                },
            ],
        );

        let template = AnimationTemplate {
            id: "zoom_in".to_string(),
            name: "Zoom In".to_string(),
            kind: AnimationKind::In,
            targets: vec![AnimationTarget::All],
            default_duration: 0.5,
            min_duration: 0.1,
            max_duration: 3.0,
            tracks,
        };

        let clip = Clip::blank(
            "test_clip",
            "track_1",
            crate::model::ClipKind::Video,
            "test_clip",
            0.0,
            5.0,
        );
        let commands = template.generate_commands(&clip, 5.0, 0.5);
        assert_eq!(commands.len(), 2);
        if let Command::SetClipKey {
            property,
            at,
            value,
            ..
        } = &commands[0]
        {
            assert_eq!(*property, KeyProperty::Scale);
            assert_eq!(*at, 0.0);
            assert!((*value - 0.05).abs() < 1e-4);
        } else {
            panic!("expected SetClipKey");
        }
    }
}
