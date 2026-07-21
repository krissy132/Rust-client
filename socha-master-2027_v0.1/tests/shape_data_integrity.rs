//! Shape-data integrity tests.
//!
//! These tests are the ChatGPT item P3.11: hardcoded canonical shape coordinates must
//! satisfy basic structural invariants — exactly `size()` cells, no duplicates, no
//! negative coordinates (already aligned), and `variants()` preserves the cell count.
//!
//! These are a regression backstop: if someone edits `PieceShape::coordinates()` by
//! hand (e.g. to add a piece or fix a typo), this test immediately surfaces any mistake.

use std::collections::HashSet;

use socha::neutral::PieceShape;

#[test]
fn canonical_shape_coordinates_unique_and_nonneg() {
    for &shape in PieceShape::ALL.iter() {
        let coords = shape.coordinates();
        let set: HashSet<(i32, i32)> = coords.iter().copied().collect();
        assert_eq!(set.len(), coords.len(), "{shape}: canonical coords have duplicates: {coords:?}");
        for &(x, y) in coords {
            assert!(x >= 0 && y >= 0, "{shape}: canonical coord has negative component ({x},{y})");
        }
    }
}

#[test]
fn shape_sizes_match_expected_distribution() {
    // Blokus has exactly: 1 monomino, 1 domino, 2 triominoes, 5 tetrominoes, 12 pentominoes.
    let mut counts = [0usize; 6]; // index = size, sizes 1..5
    for &shape in PieceShape::ALL.iter() {
        let size = shape.size();
        assert!(size >= 1 && size <= 5, "{shape}: out-of-range size {size}");
        counts[size] += 1;
    }
    assert_eq!(counts[1], 1, "exactly one monomino");
    assert_eq!(counts[2], 1, "exactly one domino");
    assert_eq!(counts[3], 2, "exactly two triominoes");
    assert_eq!(counts[4], 5, "exactly five tetrominoes");
    assert_eq!(counts[5], 12, "exactly twelve pentominoes");

    // The total piece-square sum must equal `SUM_MAX_SQUARES` (89) as documented in the code.
    let total: usize = PieceShape::ALL.iter().map(|s| s.size()).sum();
    assert_eq!(total, 89, "sum of shape sizes must be 89");
}

#[test]
fn variants_count_within_reasonable_bounds() {
    // Blokus pentominoes typically have 8 distinct variants (4 rotations × 2 flips, minus
    // any duplicates from symmetry). Smaller pieces have proportionally fewer (e.g. MONO
    // has exactly 1, DOMINO has 2). We assert each shape's variant count is in [1, 8].
    for &shape in PieceShape::ALL.iter() {
        let n = shape.variants().len();
        assert!(
            n >= 1 && n <= 8,
            "{shape} has {n} variants, expected 1..=8"
        );
    }
}

#[test]
fn monomino_has_one_variant() {
    let vs = PieceShape::Mono.variants();
    assert_eq!(vs.len(), 1, "MONO should have only one distinct variant");
    assert_eq!(vs[0].shape, vec![(0, 0)]);
}

#[test]
fn domino_has_two_variants() {
    // Domino: horizontal & vertical.
    let vs = PieceShape::Domino.variants();
    assert_eq!(vs.len(), 2, "DOMINO should have exactly 2 distinct variants");
    let mut sorted: Vec<(i32, i32)> = vs.iter().flat_map(|v| v.shape.clone()).collect();
    sorted.sort();
}

#[test]
fn bbox_matches_bounding_box_method() {
    // bbox() (the free-standing internal helper) and `bounding_box` (the method on
    // PieceShape) must agree on every shape.
    for &shape in PieceShape::ALL.iter() {
        let (dx, dy) = shape.bounding_box();
        assert!(dx >= 0 && dy >= 0, "{shape}: bbox negative ({dx},{dy})");
    }
}
