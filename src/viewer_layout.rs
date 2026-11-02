//! Geometría del visor; no depende ni de GTK ni de egui.
pub const PAGE_GAP: f64 = 22.0;
pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 2.0;
pub const THUMB_SCALE: f32 = 0.28;

#[derive(Clone, Debug)]
pub struct PageBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Debug)]
pub struct Row {
    pub top: f64,
    pub height: f64,
    pub first: usize,
    pub count: usize,
}
#[derive(Clone, Debug)]
pub struct Arrangement {
    pub columns: usize,
    pub pages: Vec<PageBox>,
    pub rows: Vec<Row>,
    pub width: f64,
    pub height: f64,
}
/// Keep document order, with no cover-page exception. Never shrink to force a spread.
pub fn arrange(sizes: &[(f32, f32)], zoom: f32, available: f64) -> Arrangement {
    let z = f64::from(zoom_clamped(zoom));
    let widths: Vec<f64> = sizes.iter().map(|s| (s.0 as f64 * z).ceil()).collect();
    let minimum = widths
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min)
        .max(1.0);
    let limit = (((available - PAGE_GAP) / (minimum + PAGE_GAP))
        .floor()
        .max(1.0) as usize)
        .min(sizes.len().max(1));
    let mut columns = 1;
    let mut slots = vec![widths.iter().copied().fold(0.0, f64::max)];
    for count in (2..=limit).rev() {
        let mut candidate = vec![0.0_f64; count];
        for (i, w) in widths.iter().enumerate() {
            candidate[i % count] = candidate[i % count].max(*w);
        }
        if candidate.iter().sum::<f64>() + PAGE_GAP * (count + 1) as f64 <= available {
            columns = count;
            slots = candidate;
            break;
        }
    }
    let width = slots.iter().sum::<f64>() + PAGE_GAP * (columns + 1) as f64;
    let mut pages = Vec::new();
    let mut rows = Vec::new();
    let mut y = PAGE_GAP;
    for c in sizes.chunks(columns) {
        let row_width = c.iter().map(|s| (f64::from(s.0) * z).ceil()).sum::<f64>()
            + PAGE_GAP * (c.len().saturating_sub(1)) as f64;
        let mut x = if columns > 1 {
            PAGE_GAP
        } else {
            (width - row_width) / 2.0
        };
        let height = c
            .iter()
            .map(|s| (f64::from(s.1) * z).ceil())
            .fold(0.0, f64::max);
        rows.push(Row {
            top: y,
            height,
            first: pages.len(),
            count: c.len(),
        });
        for (slot, &(w, h)) in c.iter().enumerate() {
            let w = (f64::from(w) * z).ceil();
            pages.push(PageBox {
                x,
                y,
                width: w,
                height: (f64::from(h) * z).ceil(),
            });
            x += if columns > 1 { slots[slot] } else { w } + PAGE_GAP;
        }
        y += height + PAGE_GAP;
    }
    Arrangement {
        columns,
        pages,
        rows,
        width,
        height: if sizes.is_empty() { 0.0 } else { y },
    }
}
impl Arrangement {
    pub fn visible(&self, first: f64, last: f64) -> Vec<usize> {
        let start = self
            .rows
            .partition_point(|r| r.top + r.height < first - 260.0);
        let end = self.rows.partition_point(|r| r.top <= last + 260.0);
        self.rows[start..end]
            .iter()
            .flat_map(|r| r.first..r.first + r.count)
            .collect()
    }
    pub fn page_at(&self, x: f64, y: f64) -> usize {
        let row = self.rows.partition_point(|r| r.top <= y).saturating_sub(1);
        let first = row.saturating_sub(1);
        let last = (row + 2).min(self.rows.len());
        self.rows[first..last]
            .iter()
            .flat_map(|r| r.first..r.first + r.count)
            .min_by(|&a, &b| {
                let distance = |p: &PageBox| {
                    let dx = (p.x - x).max(0.0).max(x - p.x - p.width);
                    let dy = (p.y - y).max(0.0).max(y - p.y - p.height);
                    dx * dx + dy * dy
                };
                distance(&self.pages[a]).total_cmp(&distance(&self.pages[b]))
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod spreads {
    use super::*;
    #[test]
    fn width_zoom_mixed_heights_and_odd_page_keep_document_order() {
        let sizes = [(612.0, 792.0), (612.0, 500.0), (400.0, 900.0)];
        let single = arrange(&sizes, 1.0, 1289.0);
        assert_eq!(single.columns, 1);
        let spread = arrange(&sizes, 1.0, 1290.0);
        assert_eq!(spread.columns, 2);
        assert_eq!(spread.pages[0].y, spread.pages[1].y);
        assert_eq!(spread.pages[2].y, 22.0 + 792.0 + 22.0);
        assert_eq!(spread.page_at(spread.pages[1].x + 50.0, 100.0), 1);
        assert_eq!(spread.visible(0.0, 100.0), vec![0, 1]);
        assert_eq!(spread.pages[2].x, spread.pages[0].x);
        assert_eq!(arrange(&sizes, 0.5, 900.0).columns, 3);
        assert_eq!(arrange(&sizes, 2.0, 1500.0).columns, 1);
        assert!(arrange(&[], 1.0, 900.0).visible(0.0, 900.0).is_empty());
    }
    #[test]
    fn indexed_spreads_match_full_scan() {
        let sizes: Vec<_> = (0..1001)
            .map(|i| {
                (
                    400.0 + (i % 3) as f32 * 70.0,
                    300.0 + (i % 7) as f32 * 113.0,
                )
            })
            .collect();
        for zoom in [0.5, 1.0, 2.0] {
            for width in [700.0, 2400.0] {
                let a = arrange(&sizes, zoom, width);
                for row in a.rows.iter().step_by(17) {
                    let first = row.top - 400.0;
                    let last = first + 800.0;
                    let expected: Vec<_> = a
                        .rows
                        .iter()
                        .filter(|r| r.top + r.height >= first - 260.0 && r.top <= last + 260.0)
                        .flat_map(|r| r.first..r.first + r.count)
                        .collect();
                    assert_eq!(a.visible(first, last), expected);
                }
            }
        }
    }
}

pub fn zoom_clamped(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(MIN_ZOOM, MAX_ZOOM)
    } else {
        1.0
    }
}

/// Coordenada Y superior de cada página, incluyendo un margen entre hojas.
pub fn page_tops(dimensions: &[(f32, f32)], zoom: f32) -> Vec<f64> {
    let z = zoom_clamped(zoom) as f64;
    let mut top = PAGE_GAP;
    dimensions
        .iter()
        .map(|&(_, height)| {
            let current = top;
            top += f64::from(height) * z + PAGE_GAP;
            current
        })
        .collect()
}

pub fn page_at(tops: &[f64], heights: &[(f32, f32)], zoom: f32, position: f64) -> usize {
    if tops.is_empty() {
        return 0;
    }
    // Ordered non-overlapping pages: inspect only the neighbours of the pointer.
    let next = tops.partition_point(|&top| top <= position);
    if next == 0 {
        return 0;
    }
    let previous = next - 1;
    if next == tops.len() {
        return previous;
    }
    let z = f64::from(zoom_clamped(zoom));
    let before = distance_to_page(position, tops[previous], f64::from(heights[previous].1) * z);
    let after = distance_to_page(position, tops[next], f64::from(heights[next].1) * z);
    if before <= after { previous } else { next }
}

fn distance_to_page(position: f64, top: f64, height: f64) -> f64 {
    if position < top {
        top - position
    } else if position > top + height {
        position - (top + height)
    } else {
        0.0
    }
}

pub fn visible_indices(
    tops: &[f64],
    heights: &[(f32, f32)],
    zoom: f32,
    first: f64,
    last: f64,
) -> Vec<usize> {
    let margin = 260.0; // Predibujar las páginas inmediatamente próximas.
    let z = f64::from(zoom_clamped(zoom));
    // Both page starts and ends are ordered; avoid scanning the whole document
    // on every scroll event. Boundary comparisons match the original viewport.
    let start = tops.partition_point(|&y| y < first - margin);
    let start = start.saturating_sub(1);
    let end = tops.partition_point(|&y| y <= last + margin);
    (start..end)
        .filter(|&i| tops[i] + f64::from(heights[i].1) * z >= first - margin)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pages_are_stacked_with_a_gap() {
        let sizes = [(612.0, 792.0), (612.0, 792.0), (612.0, 792.0)];
        let offsets = page_tops(&sizes, 1.0);
        assert_eq!(offsets, vec![22.0, 836.0, 1650.0]);
        assert_eq!(page_at(&offsets, &sizes, 1.0, 912.0), 1);
        assert_eq!(
            visible_indices(&offsets, &sizes, 1.0, 1200.0, 1500.0),
            vec![1, 2]
        );
    }
    #[test]
    fn zoom_is_bounded_and_monotonic() {
        assert_eq!(zoom_clamped(f32::NAN), 1.0);
        assert_eq!(zoom_clamped(12.0), 2.0);
        assert_eq!(zoom_clamped(0.1), 0.5);
        let sizes = [(612.0, 792.0); 2];
        assert!(page_tops(&sizes, 1.5)[1] > page_tops(&sizes, 1.0)[1]);
    }
    #[test]
    fn empty_document_does_not_crash() {
        assert!(page_tops(&[], 1.0).is_empty());
        assert_eq!(page_at(&[], &[], 1.0, 10.0), 0);
        assert!(visible_indices(&[], &[], 1.0, 0.0, 100.0).is_empty());
    }
}

#[cfg(test)]
mod lookup_regression {
    use super::*;
    #[test]
    fn indexed_lookup_matches_full_scan_on_mixed_pages_and_gaps() {
        let sizes: Vec<_> = (0..1000)
            .map(|i| (612.0, 300.0 + (i % 7) as f32 * 113.0))
            .collect();
        for zoom in [0.5, 1.0, 2.0] {
            let tops = page_tops(&sizes, zoom);
            let z = f64::from(zoom);
            for i in (0..1000).step_by(13) {
                for position in [
                    tops[i] - 500.0,
                    tops[i] - 11.0,
                    tops[i],
                    tops[i] + 300.0,
                    tops[i] + 900.0,
                ] {
                    let expected = (0..tops.len())
                        .min_by(|&a, &b| {
                            distance_to_page(position, tops[a], f64::from(sizes[a].1) * z)
                                .total_cmp(&distance_to_page(
                                    position,
                                    tops[b],
                                    f64::from(sizes[b].1) * z,
                                ))
                        })
                        .unwrap();
                    assert_eq!(page_at(&tops, &sizes, zoom, position), expected);
                    let last = position + 800.0;
                    let expected: Vec<_> = (0..tops.len())
                        .filter(|&j| {
                            tops[j] <= last + 260.0
                                && tops[j] + f64::from(sizes[j].1) * z >= position - 260.0
                        })
                        .collect();
                    assert_eq!(
                        visible_indices(&tops, &sizes, zoom, position, last),
                        expected
                    );
                }
            }
        }
    }
}

/// Visible interval in a monotonic vertical widget list. Query current GTK
/// allocations lazily: no cached CSS heights to become stale after resizing.
/// The caller supplies its fallback bounds before allocation and any prefetch margin.
pub fn ordered_visible_range(
    count: usize,
    first: f64,
    last: f64,
    mut bounds: impl FnMut(usize) -> (f64, f64),
) -> std::ops::Range<usize> {
    let (mut lo, mut hi) = (0, count);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if bounds(mid).1 < first {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    let start = lo;
    hi = count;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if bounds(mid).0 <= last {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    start..lo
}
#[cfg(test)]
mod ordered_widgets {
    use super::*;
    #[test]
    fn mixed_heights_gaps_and_exact_edges_match_the_original_full_scan() {
        let mut y = 12.0;
        let widgets: Vec<_> = (0..1001)
            .map(|i| {
                let h = 100.0 + (i % 7) as f64 * 37.0;
                let p = (y, y + h);
                y += h + 21.0;
                p
            })
            .collect();
        for &(a, b) in widgets.iter().step_by(13) {
            for top in [a - 250.0, a, b, b + 1.0] {
                let bottom = top + 850.0;
                let expected: Vec<_> = widgets
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.0 <= bottom && p.1 >= top)
                    .map(|(i, _)| i)
                    .collect();
                assert_eq!(
                    ordered_visible_range(widgets.len(), top, bottom, |i| widgets[i])
                        .collect::<Vec<_>>(),
                    expected
                );
            }
        }
        assert_eq!(
            ordered_visible_range(0, 0.0, 800.0, |_| unreachable!()),
            0..0
        );
        assert_eq!(
            ordered_visible_range(widgets.len(), y + 1.0, y + 800.0, |i| widgets[i]),
            widgets.len()..widgets.len()
        );
    }
    #[test]
    fn large_sidebar_does_not_query_every_widget() {
        let mut calls = 0;
        let range = ordered_visible_range(10_000, 500_000.0, 500_800.0, |i| {
            calls += 1;
            ((i * 300) as f64, (i * 300 + 280) as f64)
        });
        assert_eq!(range, 1666..1670);
        assert!(calls < 30, "{calls} allocation queries");
    }
}
