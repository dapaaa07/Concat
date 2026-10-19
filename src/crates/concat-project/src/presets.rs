// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Built-in animation presets for clip slots (In, Out, and Loop).
//!
//! Presets are evaluated at render time *on top of* the clip's own keyframes
//! rather than baked into them:
//! - Scale and opacity are relative multipliers (`1.0` is neutral).
//! - Offsets and rotation are relative additive deltas (`0.0` is neutral).
//!
//! Durations are in seconds rather than fractions of the clip, so trimming
//! a clip preserves its animation speed rather than stretching or compressing it.

use std::f64::consts::PI;

use concat_core::animate::Ease;

use crate::model::Clip;

/// The three animation slots a clip can have.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AnimationSlotKind {
    /// Plays at the head of the clip.
    In,
    /// Plays at the tail of the clip.
    Out,
    /// Loops continuously across the clip duration.
    Loop,
}

/// A relative transform and opacity modifier evaluated at one instant.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AnimationModifier {
    /// Multiplier on scale (`1.0` = neutral / no change).
    pub scale: f64,
    /// Additive delta on horizontal offset in frame widths (`0.0` = neutral).
    pub offset_x: f64,
    /// Additive delta on vertical offset in frame heights (`0.0` = neutral).
    pub offset_y: f64,
    /// Additive delta on clockwise rotation in degrees (`0.0` = neutral).
    pub rotation: f64,
    /// Multiplier on opacity in `0.0..=1.0` (`1.0` = neutral).
    pub opacity: f64,
}

impl Default for AnimationModifier {
    fn default() -> Self {
        AnimationModifier::IDENTITY
    }
}

impl AnimationModifier {
    /// The identity modifier: scale and opacity factor 1.0, offset and rotation delta 0.0.
    pub const IDENTITY: AnimationModifier = AnimationModifier {
        scale: 1.0,
        offset_x: 0.0,
        offset_y: 0.0,
        rotation: 0.0,
        opacity: 1.0,
    };

    /// Combines this modifier with another: factors multiply, deltas add.
    pub fn combine(self, other: AnimationModifier) -> AnimationModifier {
        AnimationModifier {
            scale: self.scale * other.scale,
            offset_x: self.offset_x + other.offset_x,
            offset_y: self.offset_y + other.offset_y,
            rotation: self.rotation + other.rotation,
            opacity: self.opacity * other.opacity,
        }
    }
}

/// Metadata for a built-in animation preset.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PresetMeta {
    /// The unique preset identifier, e.g. "fade-in", "zoom-in".
    pub id: &'static str,
    /// Human-readable label for UI display.
    pub label: &'static str,
    /// Which slot this preset belongs to.
    pub kind: AnimationSlotKind,
}

const IN_PRESETS: &[PresetMeta] = &[
    PresetMeta { id: "fade-in", label: "Fade In", kind: AnimationSlotKind::In },
    PresetMeta { id: "zoom-in", label: "Zoom In", kind: AnimationSlotKind::In },
    PresetMeta { id: "slide-left", label: "Slide Left", kind: AnimationSlotKind::In },
    PresetMeta { id: "slide-right", label: "Slide Right", kind: AnimationSlotKind::In },
    PresetMeta { id: "slide-up", label: "Slide Up", kind: AnimationSlotKind::In },
    PresetMeta { id: "slide-down", label: "Slide Down", kind: AnimationSlotKind::In },
    PresetMeta { id: "spin-in", label: "Spin In", kind: AnimationSlotKind::In },
    PresetMeta { id: "bounce-in", label: "Bounce In", kind: AnimationSlotKind::In },
];

const OUT_PRESETS: &[PresetMeta] = &[
    PresetMeta { id: "fade-out", label: "Fade Out", kind: AnimationSlotKind::Out },
    PresetMeta { id: "zoom-out", label: "Zoom Out", kind: AnimationSlotKind::Out },
    PresetMeta { id: "slide-left", label: "Slide Left", kind: AnimationSlotKind::Out },
    PresetMeta { id: "slide-right", label: "Slide Right", kind: AnimationSlotKind::Out },
    PresetMeta { id: "slide-up", label: "Slide Up", kind: AnimationSlotKind::Out },
    PresetMeta { id: "slide-down", label: "Slide Down", kind: AnimationSlotKind::Out },
    PresetMeta { id: "spin-out", label: "Spin Out", kind: AnimationSlotKind::Out },
];

const LOOP_PRESETS: &[PresetMeta] = &[
    PresetMeta { id: "pulse", label: "Pulse", kind: AnimationSlotKind::Loop },
    PresetMeta { id: "spin", label: "Spin", kind: AnimationSlotKind::Loop },
    PresetMeta { id: "bounce", label: "Bounce", kind: AnimationSlotKind::Loop },
    PresetMeta { id: "blink", label: "Blink", kind: AnimationSlotKind::Loop },
];

/// Returns the list of all built-in presets for the specified slot kind.
pub fn builtin_presets(kind: AnimationSlotKind) -> &'static [PresetMeta] {
    match kind {
        AnimationSlotKind::In => IN_PRESETS,
        AnimationSlotKind::Out => OUT_PRESETS,
        AnimationSlotKind::Loop => LOOP_PRESETS,
    }
}

/// Evaluates a preset at normalized progress `p` in `0.0..=1.0`.
pub fn evaluate_preset(kind: AnimationSlotKind, preset: &str, p: f64) -> AnimationModifier {
    let p = p.clamp(0.0, 1.0);
    match kind {
        AnimationSlotKind::In => evaluate_in_preset(preset, p),
        AnimationSlotKind::Out => evaluate_out_preset(preset, p),
        AnimationSlotKind::Loop => evaluate_loop_preset(preset, p),
    }
}

fn evaluate_in_preset(preset: &str, p: f64) -> AnimationModifier {
    let eased = Ease::OUT.apply(p);
    match preset {
        "fade-in" => AnimationModifier {
            opacity: eased,
            ..AnimationModifier::IDENTITY
        },
        "zoom-in" => AnimationModifier {
            scale: eased,
            opacity: eased,
            ..AnimationModifier::IDENTITY
        },
        "slide-left" => AnimationModifier {
            offset_x: 1.0 - eased,
            opacity: 1.0,
            ..AnimationModifier::IDENTITY
        },
        "slide-right" => AnimationModifier {
            offset_x: -(1.0 - eased),
            opacity: 1.0,
            ..AnimationModifier::IDENTITY
        },
        "slide-up" => AnimationModifier {
            offset_y: 1.0 - eased,
            opacity: 1.0,
            ..AnimationModifier::IDENTITY
        },
        "slide-down" => AnimationModifier {
            offset_y: -(1.0 - eased),
            opacity: 1.0,
            ..AnimationModifier::IDENTITY
        },
        "spin-in" => AnimationModifier {
            rotation: -360.0 * (1.0 - eased),
            scale: eased,
            opacity: eased,
            ..AnimationModifier::IDENTITY
        },
        "bounce-in" => {
            // Cubic bounce with overshoot settling at 1.0
            let scale = if p < 0.7 {
                (p / 0.7) * 1.08
            } else {
                1.08 - ((p - 0.7) / 0.3) * 0.08
            };
            AnimationModifier {
                scale,
                opacity: (p / 0.3).min(1.0),
                ..AnimationModifier::IDENTITY
            }
        }
        _ => AnimationModifier::IDENTITY,
    }
}

fn evaluate_out_preset(preset: &str, p: f64) -> AnimationModifier {
    let eased = Ease::IN.apply(p);
    match preset {
        "fade-out" => AnimationModifier {
            opacity: 1.0 - eased,
            ..AnimationModifier::IDENTITY
        },
        "zoom-out" => AnimationModifier {
            scale: 1.0 - eased,
            opacity: 1.0 - eased,
            ..AnimationModifier::IDENTITY
        },
        "slide-left" => AnimationModifier {
            offset_x: -eased,
            opacity: 1.0,
            ..AnimationModifier::IDENTITY
        },
        "slide-right" => AnimationModifier {
            offset_x: eased,
            opacity: 1.0,
            ..AnimationModifier::IDENTITY
        },
        "slide-up" => AnimationModifier {
            offset_y: -eased,
            opacity: 1.0,
            ..AnimationModifier::IDENTITY
        },
        "slide-down" => AnimationModifier {
            offset_y: eased,
            opacity: 1.0,
            ..AnimationModifier::IDENTITY
        },
        "spin-out" => AnimationModifier {
            rotation: 360.0 * eased,
            scale: 1.0 - eased,
            opacity: 1.0 - eased,
            ..AnimationModifier::IDENTITY
        },
        _ => AnimationModifier::IDENTITY,
    }
}

fn evaluate_loop_preset(preset: &str, p: f64) -> AnimationModifier {
    match preset {
        "pulse" => {
            let wave = (p * 2.0 * PI).sin();
            let scale = 1.0 + wave * 0.12;
            AnimationModifier {
                scale,
                ..AnimationModifier::IDENTITY
            }
        }
        "spin" => AnimationModifier {
            rotation: p * 360.0,
            ..AnimationModifier::IDENTITY
        },
        "bounce" => {
            let wave = (p * PI).sin();
            AnimationModifier {
                offset_y: -wave * 0.1,
                ..AnimationModifier::IDENTITY
            }
        }
        "blink" => {
            let wave = (p * 2.0 * PI).cos();
            let opacity = 0.65 + wave * 0.35;
            AnimationModifier {
                opacity,
                ..AnimationModifier::IDENTITY
            }
        }
        _ => AnimationModifier::IDENTITY,
    }
}

/// Evaluates all active animation slots on a clip at a given local playback time (in seconds).
///
/// If an animation slot is absent, it contributes the identity modifier.
/// When In and Out animations are both present and their durations sum to more
/// than the clip's duration, each is clamped to half of the clip's duration.
pub fn evaluate_clip_animations(clip: &Clip, local_seconds: f64) -> AnimationModifier {
    let duration = clip.duration.max(0.01);
    let t = local_seconds.clamp(0.0, duration);

    let max_half = duration / 2.0;
    let has_both = clip.animation_in.is_some() && clip.animation_out.is_some();

    // 1. In animation
    let in_modifier = if let Some(slot) = &clip.animation_in {
        let dur = if has_both {
            slot.duration.min(max_half).max(0.01)
        } else {
            slot.duration.min(duration).max(0.01)
        };
        if t < dur {
            let p = (t / dur).clamp(0.0, 1.0);
            evaluate_preset(AnimationSlotKind::In, &slot.preset, p)
        } else {
            AnimationModifier::IDENTITY
        }
    } else {
        AnimationModifier::IDENTITY
    };

    // 2. Out animation
    let out_modifier = if let Some(slot) = &clip.animation_out {
        let dur = if has_both {
            slot.duration.min(max_half).max(0.01)
        } else {
            slot.duration.min(duration).max(0.01)
        };
        let out_start = (duration - dur).max(0.0);
        if t >= out_start {
            let p = if dur > 0.0 {
                ((t - out_start) / dur).clamp(0.0, 1.0)
            } else {
                1.0
            };
            evaluate_preset(AnimationSlotKind::Out, &slot.preset, p)
        } else {
            AnimationModifier::IDENTITY
        }
    } else {
        AnimationModifier::IDENTITY
    };

    // 3. Loop animation
    let loop_modifier = if let Some(slot) = &clip.animation_loop {
        let cycle_dur = slot.duration.max(0.05);
        let p = (t % cycle_dur) / cycle_dur;
        evaluate_preset(AnimationSlotKind::Loop, &slot.preset, p)
    } else {
        AnimationModifier::IDENTITY
    };

    in_modifier.combine(out_modifier).combine(loop_modifier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ClipAnimation;

    #[test]
    fn builtin_presets_list_all_slots() {
        assert_eq!(builtin_presets(AnimationSlotKind::In).len(), 8);
        assert_eq!(builtin_presets(AnimationSlotKind::Out).len(), 7);
        assert_eq!(builtin_presets(AnimationSlotKind::Loop).len(), 4);
    }

    #[test]
    fn in_presets_start_transformed_and_end_at_identity() {
        // At progress 0, fade-in is transparent
        let start = evaluate_preset(AnimationSlotKind::In, "fade-in", 0.0);
        assert_eq!(start.opacity, 0.0);
        assert_eq!(start.scale, 1.0);

        // At progress 1, fade-in arrives at identity
        let end = evaluate_preset(AnimationSlotKind::In, "fade-in", 1.0);
        assert_eq!(end.opacity, 1.0);
        assert_eq!(end.scale, 1.0);
        assert_eq!(end.offset_x, 0.0);
        assert_eq!(end.offset_y, 0.0);
        assert_eq!(end.rotation, 0.0);
    }

    #[test]
    fn out_presets_start_at_identity_and_end_transformed() {
        // At progress 0, zoom-out is at identity
        let start = evaluate_preset(AnimationSlotKind::Out, "zoom-out", 0.0);
        assert_eq!(start.scale, 1.0);
        assert_eq!(start.opacity, 1.0);

        // At progress 1, zoom-out reaches 0 scale and 0 opacity
        let end = evaluate_preset(AnimationSlotKind::Out, "zoom-out", 1.0);
        assert_eq!(end.scale, 0.0);
        assert_eq!(end.opacity, 0.0);
    }

    #[test]
    fn loop_pulse_oscillates_scale() {
        let mid = evaluate_preset(AnimationSlotKind::Loop, "pulse", 0.25);
        assert!(mid.scale > 1.05);
        let rest = evaluate_preset(AnimationSlotKind::Loop, "pulse", 0.0);
        assert!((rest.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn modifiers_combine_factors_and_deltas() {
        let m1 = AnimationModifier {
            scale: 2.0,
            offset_x: 0.1,
            offset_y: -0.2,
            rotation: 45.0,
            opacity: 0.5,
        };
        let m2 = AnimationModifier {
            scale: 0.5,
            offset_x: 0.2,
            offset_y: 0.3,
            rotation: 45.0,
            opacity: 0.8,
        };
        let combined = m1.combine(m2);
        assert_eq!(combined.scale, 1.0);
        assert!((combined.offset_x - 0.3).abs() < 1e-9);
        assert!((combined.offset_y - 0.1).abs() < 1e-9);
        assert_eq!(combined.rotation, 90.0);
        assert_eq!(combined.opacity, 0.4);
    }

    #[test]
    fn evaluate_clip_animations_respects_durations_in_seconds() {
        let mut clip = Clip::blank("c1", "t1", crate::model::ClipKind::Video, "test", 0.0, 10.0);
        clip.animation_in = Some(ClipAnimation::new("fade-in", 2.0));
        clip.animation_out = Some(ClipAnimation::new("fade-out", 1.0));

        // At 0.0s, in-fade is at 0
        let m0 = evaluate_clip_animations(&clip, 0.0);
        assert_eq!(m0.opacity, 0.0);

        // At 1.0s, in-fade is at 50%
        let m1 = evaluate_clip_animations(&clip, 1.0);
        assert!(m1.opacity > 0.0 && m1.opacity < 1.0);

        // At 2.0s to 9.0s (resting interval), modifier is identity
        let m_mid = evaluate_clip_animations(&clip, 5.0);
        assert_eq!(m_mid, AnimationModifier::IDENTITY);

        // At 9.0s, out-fade begins (starts at opacity 1.0)
        let m9 = evaluate_clip_animations(&clip, 9.0);
        assert!((m9.opacity - 1.0).abs() < 1e-6);

        // At 9.5s, out-fade is halfway
        let m9_5 = evaluate_clip_animations(&clip, 9.5);
        assert!(m9_5.opacity > 0.0 && m9_5.opacity < 1.0);

        // At 10.0s, out-fade is at 0
        let m10 = evaluate_clip_animations(&clip, 10.0);
        assert_eq!(m10.opacity, 0.0);
    }

    #[test]
    fn trimming_clip_preserves_animation_speed() {
        let mut clip10 = Clip::blank("c1", "t1", crate::model::ClipKind::Video, "10s", 0.0, 10.0);
        clip10.animation_in = Some(ClipAnimation::new("fade-in", 1.0));

        let mut clip3 = Clip::blank("c2", "t1", crate::model::ClipKind::Video, "3s", 0.0, 3.0);
        clip3.animation_in = Some(ClipAnimation::new("fade-in", 1.0));

        // At 0.5s into either clip, the in-fade is at the exact same progress because time is in seconds!
        let m10 = evaluate_clip_animations(&clip10, 0.5);
        let m3 = evaluate_clip_animations(&clip3, 0.5);
        assert_eq!(m10.opacity, m3.opacity);
    }
}
