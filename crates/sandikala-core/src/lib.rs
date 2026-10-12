//! Headless musical document types and validation for the Q1 schema.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SCHEMA_VERSION: &str = "0.1.0-q1";
pub const MAX_RATIONAL_COMPONENT: u64 = 1_000_000;
pub const MAX_PARTS: usize = 64;
pub const MAX_VOICES: usize = 128;
pub const MAX_DOMAINS: usize = 128;
pub const MAX_CONTEXTS: usize = 128;
pub const MAX_VIEWS: usize = 128;
pub const MAX_EVENTS: usize = 4_096;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScoreDocument {
    pub schema_version: String,
    pub id: String,
    pub metadata: Metadata,
    pub parts: Vec<Part>,
    pub voices: Vec<Voice>,
    pub time_domains: Vec<TimeDomain>,
    pub pitch_systems: Vec<PitchSystem>,
    pub events: Vec<Event>,
    pub relations: Vec<Value>,
    pub notation_views: Vec<NotationView>,
    pub extensions: Extensions,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Metadata {
    pub title: String,
    pub composer: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub instrument_ref: InstrumentRef,
    pub voice_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstrumentRef {
    pub id: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Voice {
    pub id: String,
    pub part_id: String,
    pub time_domain_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum TimeDomain {
    Cycle {
        id: String,
        unit: String,
        cycle_length: u32,
        cycle_count: u32,
    },
    Metric {
        id: String,
        unit: String,
        length: Rational,
    },
}

impl TimeDomain {
    pub fn id(&self) -> &str {
        match self {
            Self::Cycle { id, .. } | Self::Metric { id, .. } => id,
        }
    }

    pub fn upper_extent(&self) -> Result<Rational, CoreError> {
        match self {
            Self::Cycle {
                cycle_length,
                cycle_count,
                ..
            } => Rational::from_i64(i64::from(*cycle_length))
                .checked_mul(Rational::from_i64(i64::from(*cycle_count))),
            Self::Metric { length, .. } => Ok(*length),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PitchSystem {
    Degree {
        id: String,
        degrees: Vec<String>,
        tuning_hz: HashMap<String, f64>,
    },
    EqualTemperament {
        id: String,
        steps_per_octave: u32,
        reference: EqualTemperamentReference,
    },
}

impl PitchSystem {
    pub fn id(&self) -> &str {
        match self {
            Self::Degree { id, .. } | Self::EqualTemperament { id, .. } => id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EqualTemperamentReference {
    pub letter: PitchLetter,
    pub octave: i32,
    pub frequency_hz: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PitchLetter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventAnchor {
    pub domain_id: String,
    pub position: Rational,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Event {
    Rest {
        id: String,
        voice_id: String,
        anchor: EventAnchor,
        extent: Rational,
        payload: EmptyPayload,
        extensions: Extensions,
    },
    Sound {
        id: String,
        voice_id: String,
        anchor: EventAnchor,
        extent: Rational,
        payload: SoundPayload,
        extensions: Extensions,
    },
}

impl Event {
    pub fn id(&self) -> &str {
        match self {
            Self::Rest { id, .. } | Self::Sound { id, .. } => id,
        }
    }

    pub fn voice_id(&self) -> &str {
        match self {
            Self::Rest { voice_id, .. } | Self::Sound { voice_id, .. } => voice_id,
        }
    }

    pub fn anchor(&self) -> &EventAnchor {
        match self {
            Self::Rest { anchor, .. } | Self::Sound { anchor, .. } => anchor,
        }
    }

    pub fn extent(&self) -> Rational {
        match self {
            Self::Rest { extent, .. } | Self::Sound { extent, .. } => *extent,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EmptyPayload {}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SoundPayload {
    pub pitch: Pitch,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Pitch {
    Degree {
        #[serde(rename = "contextId")]
        context_id: String,
        degree: String,
        register: i32,
    },
    Spelled {
        #[serde(rename = "contextId")]
        context_id: String,
        letter: PitchLetter,
        alteration: i32,
        octave: i32,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotationView {
    pub id: String,
    pub provider_id: String,
    pub part_ids: Vec<String>,
    pub settings: Value,
}

pub type Extensions = HashMap<String, Value>;

/// A bounded signed rational value. Constructors and arithmetic return reduced fractions.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Rational {
    pub numerator: i64,
    pub denominator: i64,
}

impl<'de> Deserialize<'de> for Rational {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Components {
            numerator: i64,
            denominator: i64,
        }

        let components = Components::deserialize(deserializer)?;
        Rational::new(components.numerator, components.denominator)
            .map_err(serde::de::Error::custom)
    }
}

impl Rational {
    pub fn new(numerator: i64, denominator: i64) -> Result<Self, CoreError> {
        if denominator <= 0
            || numerator.unsigned_abs() > MAX_RATIONAL_COMPONENT
            || denominator as u64 > MAX_RATIONAL_COMPONENT
        {
            return Err(CoreError::invalid_rational(
                "rational components are outside the Q1 bounds",
            ));
        }
        let gcd = gcd(u128::from(numerator.unsigned_abs()), denominator as u128) as i64;
        Ok(Self {
            numerator: numerator / gcd,
            denominator: denominator / gcd,
        })
    }

    pub fn from_i64(value: i64) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    pub fn validate(self) -> Result<Self, CoreError> {
        Self::new(self.numerator, self.denominator)
    }

    pub fn checked_add(self, other: Self) -> Result<Self, CoreError> {
        self.validate()?;
        other.validate()?;
        let left = i128::from(self.numerator)
            .checked_mul(i128::from(other.denominator))
            .ok_or_else(CoreError::arithmetic_limit)?;
        let right = i128::from(other.numerator)
            .checked_mul(i128::from(self.denominator))
            .ok_or_else(CoreError::arithmetic_limit)?;
        let numerator = left
            .checked_add(right)
            .ok_or_else(CoreError::arithmetic_limit)?;
        let denominator = i128::from(self.denominator)
            .checked_mul(i128::from(other.denominator))
            .ok_or_else(CoreError::arithmetic_limit)?;
        Self::from_i128(numerator, denominator)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, CoreError> {
        let negated = other
            .numerator
            .checked_neg()
            .ok_or_else(CoreError::arithmetic_limit)?;
        self.checked_add(Self {
            numerator: negated,
            denominator: other.denominator,
        })
    }

    pub fn checked_mul(self, other: Self) -> Result<Self, CoreError> {
        self.validate()?;
        other.validate()?;
        let numerator = i128::from(self.numerator)
            .checked_mul(i128::from(other.numerator))
            .ok_or_else(CoreError::arithmetic_limit)?;
        let denominator = i128::from(self.denominator)
            .checked_mul(i128::from(other.denominator))
            .ok_or_else(CoreError::arithmetic_limit)?;
        Self::from_i128(numerator, denominator)
    }

    pub fn checked_cmp(self, other: Self) -> Result<std::cmp::Ordering, CoreError> {
        self.validate()?;
        other.validate()?;
        let left = i128::from(self.numerator)
            .checked_mul(i128::from(other.denominator))
            .ok_or_else(CoreError::arithmetic_limit)?;
        let right = i128::from(other.numerator)
            .checked_mul(i128::from(self.denominator))
            .ok_or_else(CoreError::arithmetic_limit)?;
        Ok(left.cmp(&right))
    }

    fn from_i128(numerator: i128, denominator: i128) -> Result<Self, CoreError> {
        if denominator <= 0 {
            return Err(CoreError::invalid_rational("denominator must be positive"));
        }
        let gcd = gcd(numerator.unsigned_abs(), denominator as u128) as i128;
        let numerator = numerator / gcd;
        let denominator = denominator / gcd;
        if numerator.unsigned_abs() > u128::from(MAX_RATIONAL_COMPONENT)
            || denominator > i128::from(MAX_RATIONAL_COMPONENT)
            || numerator < i128::from(i64::MIN)
            || numerator > i128::from(i64::MAX)
        {
            return Err(CoreError::arithmetic_limit());
        }
        Ok(Self {
            numerator: numerator as i64,
            denominator: denominator as i64,
        })
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.max(1)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidDocument,
    InvalidReference,
    InvalidRational,
    ArithmeticLimit,
    UnsupportedSchemaVersion,
    UnsupportedCoreFeature,
    InputLimit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreError {
    pub code: ErrorCode,
    pub message: String,
    pub field: Option<String>,
    pub ids: Vec<String>,
}

impl CoreError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            field: None,
            ids: Vec::new(),
        }
    }

    fn field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    fn involving(mut self, ids: impl IntoIterator<Item = String>) -> Self {
        self.ids.extend(ids);
        self
    }

    fn invalid_rational(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidRational, message).field("rational")
    }

    fn arithmetic_limit() -> Self {
        Self::new(
            ErrorCode::ArithmeticLimit,
            "rational arithmetic exceeds the Q1 bounds",
        )
    }
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for CoreError {}

impl ScoreDocument {
    /// Decode a complete Q1 document, returning schema-specific error codes for unsupported constructs.
    pub fn parse_json(input: &str) -> Result<Self, CoreError> {
        let value: Value = serde_json::from_str(input).map_err(|error| {
            CoreError::new(ErrorCode::InvalidDocument, format!("invalid JSON: {error}"))
        })?;
        Self::from_value(value)
    }

    /// Decode a JSON value and validate it before returning the score.
    pub fn from_value(value: Value) -> Result<Self, CoreError> {
        inspect_supported_kinds(&value)?;
        inspect_rational_values(&value)?;
        let document: Self = serde_json::from_value(value).map_err(|error| {
            CoreError::new(
                ErrorCode::InvalidDocument,
                format!("document does not match the Q1 schema: {error}"),
            )
        })?;
        document.validate()?;
        Ok(document)
    }

    /// Validate the complete Q1 core contract without changing musical values.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(CoreError::new(
                ErrorCode::UnsupportedSchemaVersion,
                format!("schema version '{}' is not supported", self.schema_version),
            )
            .field("schemaVersion"));
        }
        nonempty_id(&self.id, "id")?;
        text_bound(&self.metadata.title, 256, "metadata.title")?;
        text_bound(&self.metadata.composer, 256, "metadata.composer")?;
        for (index, part) in self.parts.iter().enumerate() {
            nonempty_id(&part.id, &format!("parts[{index}].id"))?;
            nonempty_id(
                &part.instrument_ref.id,
                &format!("parts[{index}].instrumentRef.id"),
            )?;
            nonempty_id(
                &part.instrument_ref.version,
                &format!("parts[{index}].instrumentRef.version"),
            )?;
        }
        for (index, voice) in self.voices.iter().enumerate() {
            nonempty_id(&voice.id, &format!("voices[{index}].id"))?;
            nonempty_id(&voice.part_id, &format!("voices[{index}].partId"))?;
            nonempty_id(
                &voice.time_domain_id,
                &format!("voices[{index}].timeDomainId"),
            )?;
        }
        for (index, domain) in self.time_domains.iter().enumerate() {
            nonempty_id(domain.id(), &format!("timeDomains[{index}].id"))?;
        }
        for (index, context) in self.pitch_systems.iter().enumerate() {
            nonempty_id(context.id(), &format!("pitchSystems[{index}].id"))?;
        }
        for (index, view) in self.notation_views.iter().enumerate() {
            nonempty_id(&view.id, &format!("notationViews[{index}].id"))?;
            nonempty_id(
                &view.provider_id,
                &format!("notationViews[{index}].providerId"),
            )?;
        }
        enforce_limits(self)?;
        unique(self.parts.iter().map(|item| item.id.as_str()), "parts.id")?;
        unique(self.voices.iter().map(|item| item.id.as_str()), "voices.id")?;
        unique(
            self.time_domains.iter().map(TimeDomain::id),
            "timeDomains.id",
        )?;
        unique(
            self.pitch_systems.iter().map(PitchSystem::id),
            "pitchSystems.id",
        )?;
        unique(self.events.iter().map(Event::id), "events.id")?;
        unique(
            self.notation_views.iter().map(|item| item.id.as_str()),
            "notationViews.id",
        )?;

        if !self.relations.is_empty() {
            return Err(CoreError::new(
                ErrorCode::UnsupportedCoreFeature,
                "relations are unsupported in Q1",
            )
            .field("relations"));
        }

        let parts: HashMap<_, _> = self
            .parts
            .iter()
            .map(|part| (part.id.as_str(), part))
            .collect();
        let voices: HashMap<_, _> = self
            .voices
            .iter()
            .map(|voice| (voice.id.as_str(), voice))
            .collect();
        let domains: HashMap<_, _> = self
            .time_domains
            .iter()
            .map(|domain| (domain.id(), domain))
            .collect();
        let contexts: HashMap<_, _> = self
            .pitch_systems
            .iter()
            .map(|context| (context.id(), context))
            .collect();

        for part in &self.parts {
            let mut listed = HashSet::new();
            for voice_id in &part.voice_ids {
                if !listed.insert(voice_id.as_str()) {
                    return Err(CoreError::new(
                        ErrorCode::InvalidReference,
                        "part lists a voice more than once",
                    )
                    .field("parts.voiceIds")
                    .involving([part.id.clone(), voice_id.clone()]));
                }
                match voices.get(voice_id.as_str()) {
                    Some(voice) if voice.part_id == part.id => {}
                    Some(voice) => {
                        return Err(CoreError::new(
                            ErrorCode::InvalidReference,
                            "part and voice ownership disagree",
                        )
                        .field("parts.voiceIds")
                        .involving([part.id.clone(), voice.id.clone()]));
                    }
                    None => {
                        return Err(CoreError::new(
                            ErrorCode::InvalidReference,
                            "part references a missing voice",
                        )
                        .field("parts.voiceIds")
                        .involving([part.id.clone(), voice_id.clone()]));
                    }
                }
            }
        }

        for voice in &self.voices {
            let Some(part) = parts.get(voice.part_id.as_str()) else {
                return Err(CoreError::new(
                    ErrorCode::InvalidReference,
                    "voice references a missing part",
                )
                .field("voices.partId")
                .involving([voice.id.clone(), voice.part_id.clone()]));
            };
            if !part.voice_ids.iter().any(|id| id == &voice.id) {
                return Err(CoreError::new(
                    ErrorCode::InvalidReference,
                    "voice is not listed by its owning part",
                )
                .field("voices.partId")
                .involving([voice.id.clone(), part.id.clone()]));
            }
            if !domains.contains_key(voice.time_domain_id.as_str()) {
                return Err(CoreError::new(
                    ErrorCode::InvalidReference,
                    "voice references a missing time domain",
                )
                .field("voices.timeDomainId")
                .involving([voice.id.clone(), voice.time_domain_id.clone()]));
            }
        }

        for domain in &self.time_domains {
            match domain {
                TimeDomain::Cycle {
                    id,
                    unit,
                    cycle_length,
                    cycle_count,
                } => {
                    if unit != "pulse" || *cycle_length != 16 || !(1..=64).contains(cycle_count) {
                        return Err(CoreError::new(
                            ErrorCode::InvalidDocument,
                            "cycle domain must use 16 pulses and contain 1 through 64 cycles",
                        )
                        .field("timeDomains")
                        .involving([id.clone()]));
                    }
                }
                TimeDomain::Metric { id, unit, length } => {
                    validate_rational(*length, "timeDomains.length")?;
                    if unit != "quarter" || length.numerator <= 0 {
                        return Err(CoreError::new(
                            ErrorCode::InvalidDocument,
                            "metric domain requires positive quarter-note length",
                        )
                        .field("timeDomains.length")
                        .involving([id.clone()]));
                    }
                }
            }
        }

        for context in &self.pitch_systems {
            match context {
                PitchSystem::Degree {
                    id,
                    degrees,
                    tuning_hz,
                } => {
                    unique(degrees.iter().map(String::as_str), "pitchSystems.degrees")?;
                    for degree in degrees {
                        if !tuning_hz
                            .get(degree)
                            .is_some_and(|hz| hz.is_finite() && *hz > 0.0)
                        {
                            return Err(CoreError::new(
                                ErrorCode::InvalidDocument,
                                "each declared degree needs a finite positive base-register tuning",
                            )
                            .field("pitchSystems.tuningHz")
                            .involving([id.clone(), degree.clone()]));
                        }
                    }
                    if tuning_hz.keys().any(|degree| !degrees.contains(degree)) {
                        return Err(CoreError::new(
                            ErrorCode::InvalidDocument,
                            "tuningHz contains a key not declared in degrees",
                        )
                        .field("pitchSystems.tuningHz")
                        .involving([id.clone()]));
                    }
                }
                PitchSystem::EqualTemperament {
                    id,
                    steps_per_octave,
                    reference,
                } => {
                    if *steps_per_octave != 12
                        || reference.letter != PitchLetter::A
                        || reference.octave != 4
                        || reference.frequency_hz != 440.0
                    {
                        return Err(CoreError::new(
                            ErrorCode::InvalidDocument,
                            "Q1 equal temperament is 12-tone with A4=440 Hz",
                        )
                        .field("pitchSystems")
                        .involving([id.clone()]));
                    }
                }
            }
        }

        for view in &self.notation_views {
            for part_id in &view.part_ids {
                if !parts.contains_key(part_id.as_str()) {
                    return Err(CoreError::new(
                        ErrorCode::InvalidReference,
                        "notation view references a missing part",
                    )
                    .field("notationViews.partIds")
                    .involving([view.id.clone(), part_id.clone()]));
                }
            }
        }

        let mut per_voice: HashMap<&str, Vec<(&Event, Rational, Rational)>> = HashMap::new();
        for event in &self.events {
            let Some(voice) = voices.get(event.voice_id()) else {
                return Err(CoreError::new(
                    ErrorCode::InvalidReference,
                    "event references a missing voice",
                )
                .field("events.voiceId")
                .involving([event.id().to_owned(), event.voice_id().to_owned()]));
            };
            let anchor = event.anchor();
            if anchor.domain_id != voice.time_domain_id {
                return Err(CoreError::new(
                    ErrorCode::InvalidReference,
                    "event anchor domain must match its voice domain",
                )
                .field("events.anchor.domainId")
                .involving([
                    event.id().to_owned(),
                    voice.id.clone(),
                    anchor.domain_id.clone(),
                ]));
            }
            validate_rational(anchor.position, "events.anchor.position")?;
            let extent = event.extent();
            validate_rational(extent, "events.extent")?;
            if extent.numerator <= 0 {
                return Err(
                    CoreError::invalid_rational("event extent must be greater than zero")
                        .field("events.extent")
                        .involving([event.id().to_owned()]),
                );
            }
            let domain = domains[voice.time_domain_id.as_str()];
            if matches!(domain, TimeDomain::Cycle { .. }) && anchor.position.numerator < 0 {
                return Err(CoreError::invalid_rational(
                    "cycle event positions must be nonnegative",
                )
                .field("events.anchor.position")
                .involving([event.id().to_owned()]));
            }
            let end = anchor.position.checked_add(extent)?;
            if end.checked_cmp(domain.upper_extent()?)?.is_gt() {
                return Err(CoreError::new(
                    ErrorCode::InvalidDocument,
                    "event endpoint exceeds the declared time-domain extent",
                )
                .field("events.extent")
                .involving([event.id().to_owned(), voice.time_domain_id.clone()]));
            }
            if let Event::Sound { payload, .. } = event {
                validate_pitch(
                    &payload.pitch,
                    contexts.get(payload.pitch.context_id()).copied(),
                    event.id(),
                )?;
            }
            per_voice
                .entry(&voice.id)
                .or_default()
                .push((event, anchor.position, end));
        }
        for (voice_id, events) in &mut per_voice {
            events.sort_by(|left, right| {
                left.1
                    .checked_cmp(right.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for pair in events.windows(2) {
                if pair[1].1.checked_cmp(pair[0].2)?.is_lt() {
                    return Err(CoreError::new(
                        ErrorCode::InvalidDocument,
                        "event extents overlap in one voice",
                    )
                    .field("events")
                    .involving([
                        pair[0].0.id().to_owned(),
                        pair[1].0.id().to_owned(),
                        (*voice_id).to_owned(),
                    ]));
                }
            }
        }
        Ok(())
    }
}

/// Compare positions only when both anchors refer to the same time domain.
pub fn compare_anchors(
    left: &EventAnchor,
    right: &EventAnchor,
) -> Result<std::cmp::Ordering, CoreError> {
    if left.domain_id != right.domain_id {
        return Err(CoreError::new(
            ErrorCode::InvalidReference,
            "positions in different time domains cannot be compared without a mapping",
        )
        .field("anchor.domainId")
        .involving([left.domain_id.clone(), right.domain_id.clone()]));
    }
    left.position.checked_cmp(right.position)
}

fn inspect_supported_kinds(value: &Value) -> Result<(), CoreError> {
    let Some(root) = value.as_object() else {
        return Err(CoreError::new(
            ErrorCode::InvalidDocument,
            "document root must be an object",
        ));
    };
    match root.get("schemaVersion").and_then(Value::as_str) {
        Some(SCHEMA_VERSION) => {}
        Some(version) => {
            return Err(CoreError::new(
                ErrorCode::UnsupportedSchemaVersion,
                format!("schema version '{version}' is not supported"),
            )
            .field("schemaVersion"));
        }
        None => {
            return Err(CoreError::new(
                ErrorCode::InvalidDocument,
                "schemaVersion must be a string",
            )
            .field("schemaVersion"));
        }
    }
    if root
        .get("relations")
        .and_then(Value::as_array)
        .is_some_and(|items| !items.is_empty())
    {
        return Err(CoreError::new(
            ErrorCode::UnsupportedCoreFeature,
            "relations are unsupported in Q1",
        )
        .field("relations"));
    }
    for (field, allowed) in [
        ("timeDomains", &["cycle", "metric"][..]),
        ("pitchSystems", &["degree", "equalTemperament"][..]),
        ("events", &["rest", "sound"][..]),
    ] {
        if let Some(items) = root.get(field).and_then(Value::as_array) {
            for (index, item) in items.iter().enumerate() {
                if let Some(kind) = item.get("kind").and_then(Value::as_str)
                    && !allowed.contains(&kind)
                {
                    return Err(CoreError::new(
                        ErrorCode::UnsupportedCoreFeature,
                        format!("unsupported {field} kind '{kind}'"),
                    )
                    .field(format!("{field}[{index}].kind")));
                }
            }
        }
    }
    if let Some(events) = root.get("events").and_then(Value::as_array) {
        for (index, event) in events.iter().enumerate() {
            if let Some(pitch) = event
                .get("payload")
                .and_then(|payload| payload.get("pitch"))
                && let Some(kind) = pitch.get("kind").and_then(Value::as_str)
                && !["degree", "spelled"].contains(&kind)
            {
                return Err(CoreError::new(
                    ErrorCode::UnsupportedCoreFeature,
                    format!("unsupported pitch kind '{kind}'"),
                )
                .field(format!("events[{index}].payload.pitch.kind")));
            }
        }
    }
    Ok(())
}

fn inspect_rational_values(value: &Value) -> Result<(), CoreError> {
    let Some(root) = value.as_object() else {
        return Ok(());
    };
    if let Some(domains) = root.get("timeDomains").and_then(Value::as_array) {
        for (index, domain) in domains.iter().enumerate() {
            if domain.get("kind").and_then(Value::as_str) == Some("metric")
                && let Some(length) = domain.get("length")
            {
                check_rational_components(length, &format!("timeDomains[{index}].length"))?;
            }
        }
    }
    if let Some(events) = root.get("events").and_then(Value::as_array) {
        for (index, event) in events.iter().enumerate() {
            if let Some(position) = event.pointer("/anchor/position") {
                check_rational_components(position, &format!("events[{index}].anchor.position"))?;
            }
            if let Some(extent) = event.get("extent") {
                check_rational_components(extent, &format!("events[{index}].extent"))?;
            }
        }
    }
    Ok(())
}

fn check_rational_components(value: &Value, field: &str) -> Result<(), CoreError> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    let (Some(numerator), Some(denominator)) = (object.get("numerator"), object.get("denominator"))
    else {
        return Ok(());
    };
    let numerator = numerator.as_i64().ok_or_else(|| {
        CoreError::invalid_rational("numerator must be a bounded integer").field(field)
    })?;
    let denominator = denominator.as_i64().ok_or_else(|| {
        CoreError::invalid_rational("denominator must be a bounded integer").field(field)
    })?;
    Rational::new(numerator, denominator)
        .map(|_| ())
        .map_err(|error| error.field(field))
}

impl Pitch {
    fn context_id(&self) -> &str {
        match self {
            Self::Degree { context_id, .. } | Self::Spelled { context_id, .. } => context_id,
        }
    }
}

fn validate_pitch(
    pitch: &Pitch,
    context: Option<&PitchSystem>,
    event_id: &str,
) -> Result<(), CoreError> {
    match (pitch, context) {
        (
            Pitch::Degree {
                context_id,
                degree,
                register,
            },
            Some(PitchSystem::Degree { id, degrees, .. }),
        ) => {
            if context_id != id || !degrees.iter().any(|value| value == degree) {
                return Err(CoreError::new(
                    ErrorCode::InvalidReference,
                    "degree pitch must name a declared degree in its context",
                )
                .field("events.payload.pitch.degree")
                .involving([event_id.to_owned(), context_id.clone()]));
            }
            if !(-32..=32).contains(register) {
                return Err(CoreError::new(
                    ErrorCode::InvalidDocument,
                    "degree register must be between -32 and 32",
                )
                .field("events.payload.pitch.register")
                .involving([event_id.to_owned()]));
            }
        }
        (
            Pitch::Spelled {
                context_id,
                alteration,
                octave,
                ..
            },
            Some(PitchSystem::EqualTemperament { id, .. }),
        ) => {
            if context_id != id {
                return Err(CoreError::new(
                    ErrorCode::InvalidReference,
                    "spelled pitch context does not match its pitch system",
                )
                .field("events.payload.pitch.contextId")
                .involving([event_id.to_owned(), context_id.clone()]));
            }
            if !(-2..=2).contains(alteration) || !(-1..=9).contains(octave) {
                return Err(CoreError::new(
                    ErrorCode::InvalidDocument,
                    "spelled pitch alteration or octave is outside the Q1 bounds",
                )
                .field("events.payload.pitch")
                .involving([event_id.to_owned()]));
            }
        }
        (_, None) => {
            return Err(CoreError::new(
                ErrorCode::InvalidReference,
                "pitch references a missing context",
            )
            .field("events.payload.pitch.contextId")
            .involving([event_id.to_owned(), pitch.context_id().to_owned()]));
        }
        _ => {
            return Err(CoreError::new(
                ErrorCode::InvalidReference,
                "pitch kind does not match its context kind",
            )
            .field("events.payload.pitch.kind")
            .involving([event_id.to_owned(), pitch.context_id().to_owned()]));
        }
    }
    Ok(())
}

fn validate_rational(value: Rational, field: &str) -> Result<(), CoreError> {
    let normalized = value.validate().map_err(|error| error.field(field))?;
    if normalized != value {
        return Err(CoreError::invalid_rational("rational value must be normalized").field(field));
    }
    Ok(())
}

fn enforce_limits(document: &ScoreDocument) -> Result<(), CoreError> {
    for (name, size, max) in [
        ("parts", document.parts.len(), MAX_PARTS),
        ("voices", document.voices.len(), MAX_VOICES),
        ("timeDomains", document.time_domains.len(), MAX_DOMAINS),
        ("pitchSystems", document.pitch_systems.len(), MAX_CONTEXTS),
        ("notationViews", document.notation_views.len(), MAX_VIEWS),
        ("events", document.events.len(), MAX_EVENTS),
    ] {
        if size > max {
            return Err(CoreError::new(
                ErrorCode::InputLimit,
                format!("{name} count {size} exceeds limit {max}"),
            )
            .field(name));
        }
    }
    Ok(())
}

fn nonempty_id(value: &str, field: &str) -> Result<(), CoreError> {
    if value.is_empty() || value.chars().count() > 128 {
        return Err(CoreError::new(
            ErrorCode::InvalidDocument,
            "ID must contain 1 through 128 characters",
        )
        .field(field));
    }
    Ok(())
}

fn text_bound(value: &str, limit: usize, field: &str) -> Result<(), CoreError> {
    if value.chars().count() > limit {
        return Err(CoreError::new(
            ErrorCode::InvalidDocument,
            format!("text exceeds {limit} characters"),
        )
        .field(field));
    }
    Ok(())
}

fn unique<'a>(values: impl Iterator<Item = &'a str>, field: &str) -> Result<(), CoreError> {
    let mut seen = HashSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(
                CoreError::new(ErrorCode::InvalidDocument, "duplicate ID or token")
                    .field(field)
                    .involving([value.to_owned()]),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> ScoreDocument {
        let path = format!("{}/../../fixtures/q1/{name}", env!("CARGO_MANIFEST_DIR"));
        let json = std::fs::read_to_string(path).expect("fixture file");
        serde_json::from_str(&json).expect("fixture decodes")
    }

    #[test]
    fn both_independent_fixtures_validate_and_preserve_typed_pitch_forms() {
        let gamelan = fixture("gamelan-basic.sdlk");
        gamelan.validate().expect("gamelan fixture valid");
        assert_eq!(gamelan.events.len(), 16);
        assert!(matches!(gamelan.events[5], Event::Rest { .. }));

        let piano = fixture("piano-model.sdlk");
        piano.validate().expect("piano fixture valid");
        assert!(matches!(
            &piano.events[0],
            Event::Sound {
                payload: SoundPayload {
                    pitch: Pitch::Spelled { .. }
                },
                ..
            }
        ));
    }

    #[test]
    fn rational_arithmetic_reduces_results_and_checks_invalid_denominators() {
        assert_eq!(
            Rational::new(2, 4).expect("fraction"),
            Rational {
                numerator: 1,
                denominator: 2
            }
        );
        let result = Rational::new(1, 3)
            .expect("third")
            .checked_add(Rational::new(1, 6).expect("sixth"))
            .expect("sum");
        assert_eq!(
            result,
            Rational {
                numerator: 1,
                denominator: 2
            }
        );
        assert_eq!(
            Rational::new(5, 0).expect_err("zero denominator").code,
            ErrorCode::InvalidRational
        );
        let decoded: Rational =
            serde_json::from_str(r#"{"numerator":2,"denominator":4}"#).expect("decode fraction");
        assert_eq!(
            decoded,
            Rational {
                numerator: 1,
                denominator: 2
            }
        );
    }

    #[test]
    fn document_validation_rejects_directly_constructed_noncanonical_rationals() {
        let mut document = fixture("gamelan-basic.sdlk");
        document.events[0].anchor_mut().position = Rational {
            numerator: 2,
            denominator: 4,
        };

        assert_eq!(
            document.validate().expect_err("noncanonical rational").code,
            ErrorCode::InvalidRational
        );
    }

    #[test]
    fn strict_core_fields_and_extensions_have_opposite_policies() {
        let mut doc = fixture("gamelan-basic.sdlk");
        doc.extensions.insert(
            "vendor.example".into(),
            serde_json::json!({"opaque": [1, true, null]}),
        );
        let round_trip = serde_json::to_value(&doc).expect("serialize");
        let decoded: ScoreDocument =
            serde_json::from_value(round_trip.clone()).expect("extension preserved");
        assert_eq!(
            decoded.extensions["vendor.example"],
            serde_json::json!({"opaque": [1, true, null]})
        );

        let mut with_extra_core = round_trip;
        with_extra_core["metadata"]["ignored"] = Value::Bool(true);
        assert!(serde_json::from_value::<ScoreDocument>(with_extra_core).is_err());
    }

    #[test]
    fn rejects_bad_references_overlap_bounds_and_cycle_negative_positions() {
        let mut doc = fixture("gamelan-basic.sdlk");
        doc.events[0].anchor_mut().position = Rational::from_i64(1);
        assert_eq!(
            doc.validate().expect_err("overlap").code,
            ErrorCode::InvalidDocument
        );

        let mut doc = fixture("gamelan-basic.sdlk");
        doc.voices[0].part_id = "missing".into();
        assert_eq!(
            doc.validate().expect_err("bad ref").code,
            ErrorCode::InvalidReference
        );

        let mut doc = fixture("gamelan-basic.sdlk");
        doc.events[0].anchor_mut().position = Rational::from_i64(-1);
        assert_eq!(
            doc.validate().expect_err("negative cycle position").code,
            ErrorCode::InvalidRational
        );

        let mut doc = fixture("gamelan-basic.sdlk");
        if let Event::Sound { extent, .. } = &mut doc.events[15] {
            *extent = Rational::from_i64(2);
        }
        assert_eq!(
            doc.validate().expect_err("past extent").code,
            ErrorCode::InvalidDocument
        );

        let mut doc = fixture("gamelan-basic.sdlk");
        doc.events[1].anchor_mut().position = Rational {
            numerator: 2,
            denominator: 0,
        };
        assert_eq!(
            doc.validate().expect_err("bad denominator").code,
            ErrorCode::InvalidRational
        );

        let mut doc = fixture("gamelan-basic.sdlk");
        doc.events[1] = doc.events[0].clone();
        assert_eq!(
            doc.validate().expect_err("duplicate event id").code,
            ErrorCode::InvalidDocument
        );
    }

    impl Event {
        fn anchor_mut(&mut self) -> &mut EventAnchor {
            match self {
                Event::Rest { anchor, .. } | Event::Sound { anchor, .. } => anchor,
            }
        }
    }

    #[test]
    fn unknown_version_and_nonempty_relations_are_explicitly_unsupported() {
        let mut doc = fixture("gamelan-basic.sdlk");
        doc.schema_version = "0.2".into();
        assert_eq!(
            doc.validate().expect_err("version").code,
            ErrorCode::UnsupportedSchemaVersion
        );
        let mut doc = fixture("gamelan-basic.sdlk");
        doc.relations.push(Value::Null);
        assert_eq!(
            doc.validate().expect_err("relations").code,
            ErrorCode::UnsupportedCoreFeature
        );
    }

    #[test]
    fn parsing_reports_unknown_core_kinds_and_cross_domain_comparisons() {
        let mut value = serde_json::to_value(fixture("gamelan-basic.sdlk")).expect("fixture value");
        value["events"][0]["kind"] = Value::String("tie".into());
        let error = ScoreDocument::from_value(value).expect_err("unsupported event kind");
        assert_eq!(error.code, ErrorCode::UnsupportedCoreFeature);

        let left = EventAnchor {
            domain_id: "cycle".into(),
            position: Rational::from_i64(0),
        };
        let right = EventAnchor {
            domain_id: "metric".into(),
            position: Rational::from_i64(0),
        };
        assert_eq!(
            compare_anchors(&left, &right)
                .expect_err("unmapped domains")
                .code,
            ErrorCode::InvalidReference
        );

        let mut value = serde_json::to_value(fixture("gamelan-basic.sdlk")).expect("fixture value");
        value["events"][0]["extent"]["denominator"] = Value::from(0);
        assert_eq!(
            ScoreDocument::from_value(value)
                .expect_err("invalid rational denominator")
                .code,
            ErrorCode::InvalidRational
        );
    }
}
