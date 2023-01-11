use rust_music_theory::note::{Note, PitchClass};

fn note_number<'a, T: Into<&'a Note>>(note: T) -> u8 {
    let note = note.into();
    let pitch_class = note.pitch_class.into_u8();
    let octave = note.octave;
    pitch_class + 12 * octave
}

/// Convert a note to a frequency
pub fn note_to_freq<'a, T: Into<&'a Note>>(note: T) -> f32 {
    let a440 = note_number(&Note::new(PitchClass::A, 4));
    2f32.powf((note_number(note) as i16 - a440 as i16) as f32 / 12.0) * 440.0
}
