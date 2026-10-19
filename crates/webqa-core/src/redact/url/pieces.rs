//! Pieces of a URL whose secret shows only after decoding more than once.

use std::iter::successors;

use percent_encoding::utf8_percent_encode;

use super::super::Redactor;
use super::WRITTEN;
use super::layer::Decoded;

/// How many times a piece is decoded at most.
const DEPTH: usize = 3;

impl Redactor {
    /// `encoded` with each piece between `separators` replaced as a whole
    /// when its second or third decoding shows a secret. `None` when none
    /// does.
    pub(super) fn mask_pieces(
        &self,
        encoded: &str,
        separators: &[char],
        plus: bool,
    ) -> Option<String> {
        let pieces: Vec<(&str, Option<String>)> = encoded
            .split_inclusive(separators)
            .map(|piece| (piece, self.mask_piece(piece, separators, plus)))
            .collect();
        pieces.iter().any(|(_, masked)| masked.is_some()).then(|| {
            pieces
                .iter()
                .map(|(piece, masked)| masked.as_deref().unwrap_or(piece))
                .collect()
        })
    }

    /// The piece, keeping the separator after it, replaced when a deeper
    /// decoding shows a secret.
    fn mask_piece(&self, piece: &str, separators: &[char], plus: bool) -> Option<String> {
        let body = piece.strip_suffix(separators).unwrap_or(piece);
        let separator = piece.get(body.len()..).unwrap_or_default();
        self.mask_deep(body, plus)
            .map(|masked| format!("{masked}{separator}"))
    }

    /// The first of the second and third decodings that masking changes,
    /// masked and encoded once. Decoding stops when it changes nothing.
    fn mask_deep(&self, body: &str, plus: bool) -> Option<String> {
        successors(Some(decoded(body, plus)), |layer| {
            Some(decoded(layer, plus)).filter(|next| next != layer)
        })
        .skip(1)
        .take(DEPTH - 1)
        .find_map(|layer| self.masked(&layer))
        .map(|masked| utf8_percent_encode(&masked, WRITTEN).to_string())
    }
}

fn decoded(encoded: &str, plus: bool) -> String {
    Decoded::new(encoded, plus).text
}
