//! One-off passwords for the accounts an admin creates.
//!
//! The create-user form has no password field, so the portal has to supply one.
//! It used to send the same literal for everybody, which made every account an
//! admin created log-in-able by anyone who knew the username (the literal sat in
//! a public repository). Each account now gets its own random password, shown to
//! the admin once and marked "must change at first login".
//!
//! Randomness comes from `crypto.getRandomValues` and nothing else. When the
//! browser cannot provide it [`generate`] returns `None` and the caller refuses
//! to create the account: a weak password is worse than no account, so there is
//! deliberately no `Math.random` fallback.

/// Letters and digits that cannot be mistaken for each other when the password
/// is read aloud or copied by hand (no `I`, `O`, `l`, `0`, `1`).
const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
const DIGIT: &[u8] = b"23456789";
/// Symbols that survive a chat message, an e-mail and a shell prompt unharmed.
const SPECIAL: &[u8] = b"!@#$%&*?-_=+";

/// 20 characters. The authenc policy asks for 8 or more with an upper-case
/// letter, a lower-case letter, a digit and a symbol; every class is guaranteed
/// below, the rest of the length is drawn from all four.
pub const LENGTH: usize = 20;

/// A source of random bytes. Returns `false` when it cannot fill the buffer.
pub type FillRandom<'a> = &'a mut dyn FnMut(&mut [u8]) -> bool;

/// A uniformly random index in `0..n`, by rejection sampling so that no
/// character is likelier than another (`byte % n` alone favours the low ones
/// whenever 256 is not a multiple of `n`).
fn index_below(n: usize, fill: FillRandom<'_>) -> Option<usize> {
    debug_assert!((1..=256).contains(&n));
    let limit = 256 - (256 % n);
    // A byte at or above `limit` is discarded. The chance of that is under
    // 10 %, so 64 draws failing in a row means the source is broken, not unlucky.
    for _ in 0..64 {
        let mut byte = [0u8; 1];
        if !fill(&mut byte) {
            return None;
        }
        if usize::from(byte[0]) < limit {
            return Some(usize::from(byte[0]) % n);
        }
    }
    None
}

fn pick(alphabet: &[u8], fill: FillRandom<'_>) -> Option<char> {
    index_below(alphabet.len(), fill).map(|i| char::from(alphabet[i]))
}

/// Build a password from `fill`. Split from [`generate`] so tests can drive it
/// with a deterministic source.
pub fn build(fill: FillRandom<'_>) -> Option<String> {
    let all: Vec<u8> = [UPPER, LOWER, DIGIT, SPECIAL].concat();
    let mut chars = vec![
        pick(UPPER, fill)?,
        pick(LOWER, fill)?,
        pick(DIGIT, fill)?,
        pick(SPECIAL, fill)?,
    ];
    while chars.len() < LENGTH {
        chars.push(pick(&all, fill)?);
    }
    // The four guaranteed characters were drawn first; shuffle (Fisher-Yates) so
    // they are not always at the front.
    for i in (1..chars.len()).rev() {
        let j = index_below(i + 1, fill)?;
        chars.swap(i, j);
    }
    Some(chars.into_iter().collect())
}

/// A fresh random password, or `None` when the browser offers no secure
/// randomness.
#[cfg(target_arch = "wasm32")]
pub fn generate() -> Option<String> {
    let crypto = web_sys::window()?.crypto().ok()?;
    build(&mut |buf: &mut [u8]| crypto.get_random_values_with_u8_array(buf).is_ok())
}

/// Native builds exist only so the workspace compiles and tests run; the portal
/// is a browser app and never creates accounts from here.
#[cfg(not(target_arch = "wasm32"))]
pub fn generate() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// xorshift64*: deterministic, good enough to exercise the shape of the output.
    fn source(seed: u64) -> impl FnMut(&mut [u8]) -> bool {
        let mut state = seed.max(1);
        move |buf: &mut [u8]| {
            for b in buf.iter_mut() {
                state ^= state >> 12;
                state ^= state << 25;
                state ^= state >> 27;
                *b = (state.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 56) as u8;
            }
            true
        }
    }

    fn classes(p: &str) -> (bool, bool, bool, bool) {
        (
            p.bytes().any(|b| UPPER.contains(&b)),
            p.bytes().any(|b| LOWER.contains(&b)),
            p.bytes().any(|b| DIGIT.contains(&b)),
            p.bytes().any(|b| SPECIAL.contains(&b)),
        )
    }

    #[test]
    fn every_password_has_the_length_and_all_four_classes() {
        // The guarantee must hold for every draw, not on average.
        for seed in 1..=500u64 {
            let p = build(&mut source(seed)).expect("source never fails");
            assert_eq!(p.len(), LENGTH, "seed {seed}: {p}");
            assert_eq!(classes(&p), (true, true, true, true), "seed {seed}: {p}");
        }
    }

    #[test]
    fn only_unambiguous_characters_are_used() {
        let allowed: Vec<u8> = [UPPER, LOWER, DIGIT, SPECIAL].concat();
        for seed in 1..=200u64 {
            let p = build(&mut source(seed)).unwrap();
            assert!(p.bytes().all(|b| allowed.contains(&b)), "{p}");
            assert!(!p.contains(['I', 'O', 'l', '0', '1']), "{p}");
        }
    }

    #[test]
    fn different_draws_give_different_passwords() {
        let a = build(&mut source(1)).unwrap();
        let b = build(&mut source(2)).unwrap();
        assert_ne!(a, b);
        // And it is never the literal this module replaced.
        assert_ne!(a, "TempPass123!");
    }

    #[test]
    fn a_source_that_cannot_fill_means_no_password_at_all() {
        // No silent fallback to something weaker.
        assert_eq!(build(&mut |_: &mut [u8]| false), None);
        // Dies half way through: still None, never a short password.
        let mut calls = 0;
        let mut dies = |buf: &mut [u8]| {
            calls += 1;
            buf.fill(7);
            calls < 10
        };
        assert_eq!(build(&mut dies), None);
    }

    #[test]
    fn a_stuck_source_stops_instead_of_looping() {
        // The first draw is for the 24 upper-case letters, whose limit is 240,
        // so a source that only ever says 255 is refused every time and must end
        // in None after its retries, not hang.
        assert_eq!(
            build(&mut |buf: &mut [u8]| {
                buf.fill(255);
                true
            }),
            None
        );
    }

    #[test]
    fn rejection_sampling_has_no_modulo_bias() {
        // 24 letters: 256 % 24 = 16, so bytes 240..=255 must be refused. If they
        // were accepted, indexes 0..16 would come up 11 times for every 10 of
        // the rest.
        let mut counts = [0u32; 24];
        for byte in 0u8..=255 {
            let mut one_shot = |buf: &mut [u8]| {
                buf[0] = byte;
                true
            };
            if let Some(i) = index_below(24, &mut one_shot) {
                counts[i] += 1;
            }
        }
        assert!(counts.iter().all(|&c| c == 10), "{counts:?}");
    }

    #[test]
    fn native_builds_refuse_to_make_one() {
        #[cfg(not(target_arch = "wasm32"))]
        assert_eq!(generate(), None);
    }
}
