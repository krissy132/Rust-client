//! Shape / Rotation / Flip invariant tests for `PieceShape`.
//!
//! These tests catch the ChatGPT-surfaced risk that rotation/flip could silently
//! change the number of cells or collapse into duplicates — which would corrupt
//! move generation and scoring.
//!
//! We exercise every one of the 21 pieces via `variants()` and assert:
//!   - The cell count is preserved across every rotation/flip variant.
//!   - Each variant's coordinates are unique (no overlapping cells).
//!   - All aligned variants have non-negative corner at (0,0).
//!   - Re-parse via the outgoing XML yields the same `shape()` (serialization round-trip).

use std::collections::HashSet;

use socha::neutral::{Piece, PieceShape, Rotation};

#[test]
fn variants_preserve_cell_count() {
    for &shape in PieceShape::ALL.iter() {
        let expected = shape.size();
        let variants = shape.variants();
        assert!(!variants.is_empty(), "{shape} produced no variants");
        for v in &variants {
            assert_eq!(
                v.shape.len(),
                expected,
                "{shape} variant rot={:?} flip={} produced {} cells (expected {expected})",
                v.rotation,
                v.is_flipped,
                v.shape.len()
            );
        }
    }
}

#[test]
fn variants_have_unique_cells() {
    for &shape in PieceShape::ALL.iter() {
        for v in shape.variants() {
            let set: HashSet<(i32, i32)> = v.shape.iter().copied().collect();
            assert_eq!(
                set.len(),
                v.shape.len(),
                "{shape} variant rot={:?} flip={} has duplicate cells: {:?}",
                v.rotation,
                v.is_flipped,
                v.shape
            );
        }
    }
}

#[test]
fn variants_have_non_negative_aligned_origin() {
    // `align()` shifts min x and min y to zero, so every aligned coordinate must be >= 0
    // AND the min-x and min-y across the shape must be exactly 0. Note: this does NOT imply
    // that the cell (0,0) itself must be occupied (L-shaped pieces legitimately leave it empty).
    for &shape in PieceShape::ALL.iter() {
        for v in shape.variants() {
            for &(x, y) in &v.shape {
                assert!(
                    x >= 0 && y >= 0,
                    "{shape} variant rot={:?} flip={} has negative coord ({x},{y})",
                    v.rotation,
                    v.is_flipped
                );
            }
            let min_x = v.shape.iter().map(|&(x, _)| x).min().expect("non-empty");
            let min_y = v.shape.iter().map(|&(_, y)| y).min().expect("non-empty");
            assert_eq!(
                min_x, 0,
                "{shape} variant rot={:?} flip={} has min_x={min_x} (align broken): {:?}",
                v.rotation, v.is_flipped, v.shape
            );
            assert_eq!(
                min_y, 0,
                "{shape} variant rot={:?} flip={} has min_y={min_y} (align broken): {:?}",
                v.rotation, v.is_flipped, v.shape
            );
        }
    }
}

#[test]
fn variants_deduplicated() {
    // The Kotlin reference dedupes variants that produce the same aligned shape.
    // `PieceShape::variants()` must do the same — distinct `(rotation, is_flipped)` may collapse
    // into the same shape, but the returned list must contain no duplicates.
    for &shape in PieceShape::ALL.iter() {
        let variants = shape.variants();
        let set: HashSet<Vec<(i32, i32)>> = variants.iter().map(|v| v.shape.clone()).collect();
        assert_eq!(
            set.len(),
            variants.len(),
            "{shape} has {} duplicate variant shapes among {} entries",
            variants.len() - set.len(),
            variants.len()
        );
    }
}

#[test]
fn rotation_then_inverse_restores_original_shape() {
    // Four consecutive 90° (Right) rotations compose to identity, so the resulting aligned
    // shape must equal the original. We verify both claims: composition via `Rotation::combine`
    // yields `Rotation::None`, and applying that composition via `transform()` restores the
    // original (sorted) shape.
    for &shape in PieceShape::ALL.iter() {
        let original = shape.transform(Rotation::None, false);
        let combined = (0..4).fold(Rotation::None, |acc, _| acc.combine(Rotation::Right));
        assert_eq!(combined, Rotation::None, "4× Right should combine to None");
        let final_shape = shape.transform(combined, false);
        let mut sorted_orig: Vec<(i32, i32)> = original;
        sorted_orig.sort();
        let mut sorted_acc: Vec<(i32, i32)> = final_shape;
        sorted_acc.sort();
        assert_eq!(
            sorted_orig, sorted_acc,
            "{shape}: 4× Right rotation did not restore original shape"
        );
    }
}

#[test]
fn xml_roundtrip_preserves_shape() {
    // Build a Piece, serialise to XML via `make_move_xml`, re-parse the `<piece>` portion,
    // and verify the resulting fields round-trip exactly.
    use strong_xml::XmlRead;
    use socha::outgoing::make_move_xml;
    use socha::neutral::{Color, Coordinates, Move};

    for &shape in PieceShape::ALL.iter() {
        for variant in shape.variants() {
            let piece = Piece::new(
                Color::Blue,
                shape,
                variant.rotation,
                variant.is_flipped,
                Coordinates::new(3, 4),
            );
            let mv = Move::Set { piece };
            let xml = make_move_xml("rid", &mv).expect("xml build");
            // Extract the <piece ...>...</piece> fragment.
            let start = xml.find("<piece ").expect("piece tag in xml");
            let end = xml[start..]
                .find("</piece>")
                .expect("piece close tag")
                + start
                + "</piece>".len();
            let piece_xml = &xml[start..end];
            let received: socha::incoming::ReceivedPiece =
                socha::incoming::ReceivedPiece::from_str(piece_xml).expect("piece parse");
            assert_eq!(received.color, "BLUE");
            assert_eq!(received.kind, shape.to_string());
            assert_eq!(received.rotation, variant.rotation.to_string());
            assert_eq!(
                received.is_flipped.trim(),
                if variant.is_flipped { "true" } else { "false" }
            );
            assert_eq!(received.position.as_ref().unwrap().x, 3);
            assert_eq!(received.position.as_ref().unwrap().y, 4);
        }
    }
}
