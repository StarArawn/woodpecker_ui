/// Which contiguous slice of a uniform-extent item list is currently visible, plus
/// leading/trailing spacer sizes so a container's total content extent (and therefore its
/// scrollbar range) stays correct even though only the visible items are actually spawned.
/// Pure and ECS-free by design -- shared by `Table`'s virtualized mode and `VirtualList`,
/// neither of which need to duplicate this math.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualWindow {
    /// First visible item index (inclusive).
    pub start: usize,
    /// Last visible item index (exclusive) -- iterate `start..end`.
    pub end: usize,
    /// Extent standing in for items `[0, start)`, so the container's measured content size
    /// (and therefore its scrollbar range) is unaffected by only spawning `start..end`.
    pub lead_spacer: f32,
    /// Extent standing in for items `[end, item_count)`.
    pub trail_spacer: f32,
}

/// Computes which items in a `scroll_offset`-scrolled, `viewport_extent`-tall (or wide)
/// container of `item_count` uniform-`item_extent` items are currently visible, padded by
/// `overscan` extra items on each side (smooths fast scrolling and keyboard navigation by
/// keeping a few off-screen items already mounted).
pub fn compute_virtual_window(
    scroll_offset: f32,
    viewport_extent: f32,
    item_extent: f32,
    item_count: usize,
    overscan: usize,
) -> VirtualWindow {
    if item_count == 0 {
        return VirtualWindow {
            start: 0,
            end: 0,
            lead_spacer: 0.0,
            trail_spacer: 0.0,
        };
    }

    // A non-positive item extent makes "which items are visible" meaningless -- render
    // everything rather than divide by zero or produce a nonsensical window.
    if item_extent <= 0.0 {
        return VirtualWindow {
            start: 0,
            end: item_count,
            lead_spacer: 0.0,
            trail_spacer: 0.0,
        };
    }

    let content_extent = item_count as f32 * item_extent;
    let scroll_offset = scroll_offset.clamp(0.0, (content_extent - viewport_extent).max(0.0));

    let first_visible = (scroll_offset / item_extent).floor() as usize;
    let last_visible = ((scroll_offset + viewport_extent.max(0.0)) / item_extent).ceil() as usize;

    let start = first_visible.saturating_sub(overscan);
    let end = last_visible.saturating_add(overscan).min(item_count);
    // `end` can end up `<= start` for a zero-height viewport or `first_visible` already past
    // the last item (e.g. after `item_count` shrinks) -- collapse to an empty, valid window
    // rather than an inverted range.
    let end = end.max(start);

    VirtualWindow {
        start,
        end,
        lead_spacer: start as f32 * item_extent,
        trail_spacer: (item_count - end) as f32 * item_extent,
    }
}

/// Widens a base `overscan` (a floor, not a fixed value) to cover however far the scroll
/// offset actually moved between two rendered frames, so the visible-window computed this
/// frame still overlaps the one computed last frame -- without this, a scroll faster than the
/// base overscan (a burst of trackpad/wheel momentum between two frames is common) jumps the
/// window clean past the previously-mounted range, leaving zero overlap. Zero overlap forces
/// the whole previous batch of items to be despawned and a whole new batch spawned in the same
/// frame; the new rows then have to wait a frame for layout to catch up, which is what shows
/// up as a visible flicker while scrolling. Capped at `max_overscan` so a single extreme jump
/// (e.g. dragging a scrollbar thumb straight to the end) can't force mounting an unbounded
/// number of items in one frame.
pub fn dynamic_overscan(
    prev_scroll_offset: f32,
    scroll_offset: f32,
    item_extent: f32,
    base_overscan: usize,
    max_overscan: usize,
) -> usize {
    let delta_items = if item_extent > 0.0 {
        ((scroll_offset - prev_scroll_offset).abs() / item_extent).ceil() as usize
    } else {
        0
    };
    base_overscan
        .max(delta_items + base_overscan)
        .min(max_overscan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_list_yields_empty_window() {
        let window = compute_virtual_window(0.0, 500.0, 20.0, 0, 2);
        assert_eq!(
            window,
            VirtualWindow {
                start: 0,
                end: 0,
                lead_spacer: 0.0,
                trail_spacer: 0.0
            }
        );
    }

    #[test]
    fn non_positive_item_extent_renders_everything() {
        let window = compute_virtual_window(0.0, 500.0, 0.0, 10, 2);
        assert_eq!(window.start, 0);
        assert_eq!(window.end, 10);
        assert_eq!(window.lead_spacer, 0.0);
        assert_eq!(window.trail_spacer, 0.0);

        let window = compute_virtual_window(0.0, 500.0, -5.0, 10, 2);
        assert_eq!((window.start, window.end), (0, 10));
    }

    #[test]
    fn at_top_shows_first_items_plus_overscan() {
        // 500px viewport / 20px rows = 25 visible rows; overscan 2 trailing only (no
        // leading room to overscan into since we're already at the top).
        let window = compute_virtual_window(0.0, 500.0, 20.0, 1000, 2);
        assert_eq!(window.start, 0);
        assert_eq!(window.end, 27);
        assert_eq!(window.lead_spacer, 0.0);
        assert_eq!(window.trail_spacer, (1000 - 27) as f32 * 20.0);
    }

    #[test]
    fn scrolled_partway_windows_around_the_offset() {
        // Scrolled 505px down: first visible row floor(505/20) = 25, last visible
        // ceil((505+500)/20) = 51 (exclusive). Overscan 2 each side -> [23, 53).
        let window = compute_virtual_window(505.0, 500.0, 20.0, 1000, 2);
        assert_eq!(window.start, 23);
        assert_eq!(window.end, 53);
        assert_eq!(window.lead_spacer, 23.0 * 20.0);
        assert_eq!(window.trail_spacer, (1000 - 53) as f32 * 20.0);
    }

    #[test]
    fn scroll_offset_past_content_end_clamps() {
        // Way more than the 200 total content px (10 items * 20px) -- clamps to the max
        // valid scroll position (0, since viewport already exceeds content), not a
        // nonsensical or panicking window.
        let window = compute_virtual_window(1_000_000.0, 500.0, 20.0, 10, 2);
        assert_eq!(window.start, 0);
        assert_eq!(window.end, 10);
    }

    #[test]
    fn scroll_offset_past_content_end_with_small_viewport_clamps_to_tail() {
        // Content is 10 * 20 = 200px, viewport 50px -- max valid scroll is 150px, so an
        // enormous requested offset clamps there, showing the tail of the list.
        let window = compute_virtual_window(1_000_000.0, 50.0, 20.0, 10, 0);
        assert_eq!(window.start, 7);
        assert_eq!(window.end, 10);
        assert_eq!(window.trail_spacer, 0.0);
    }

    #[test]
    fn overscan_larger_than_item_count_still_produces_a_valid_window() {
        let window = compute_virtual_window(0.0, 100.0, 20.0, 3, 50);
        assert_eq!(window.start, 0);
        assert_eq!(window.end, 3);
        assert_eq!(window.lead_spacer, 0.0);
        assert_eq!(window.trail_spacer, 0.0);
    }

    #[test]
    fn negative_scroll_offset_clamps_to_zero() {
        let window = compute_virtual_window(-100.0, 500.0, 20.0, 1000, 0);
        assert_eq!(window.start, 0);
    }

    #[test]
    fn zero_viewport_extent_yields_a_thin_but_valid_window() {
        let window = compute_virtual_window(100.0, 0.0, 20.0, 1000, 1);
        // first_visible = 5, last_visible = ceil(100/20) = 5 -- with overscan 1: [4, 6).
        assert_eq!(window.start, 4);
        assert_eq!(window.end, 6);
    }

    #[test]
    fn dynamic_overscan_stays_at_the_floor_when_scroll_offset_is_unchanged() {
        assert_eq!(dynamic_overscan(100.0, 100.0, 20.0, 4, 64), 4);
    }

    #[test]
    fn dynamic_overscan_widens_to_cover_a_large_single_frame_jump() {
        // A 400px jump at item_extent 20 is 20 items -- overscan must widen enough that the
        // new window's start reaches back far enough to still overlap the old window's end.
        let overscan = dynamic_overscan(0.0, 400.0, 20.0, 4, 64);
        assert_eq!(overscan, 24);
    }

    #[test]
    fn dynamic_overscan_is_capped_at_max_overscan_for_an_extreme_jump() {
        // A scrollbar-thumb-to-the-end style jump implies a huge delta -- must not mount an
        // unbounded number of items in one frame.
        let overscan = dynamic_overscan(0.0, 100_000.0, 20.0, 4, 64);
        assert_eq!(overscan, 64);
    }

    #[test]
    fn dynamic_overscan_ignores_direction_of_scroll() {
        // Scrolling back up by the same large amount must widen overscan identically to
        // scrolling down -- it's the magnitude of the jump that matters, not its sign.
        let down = dynamic_overscan(0.0, 400.0, 20.0, 4, 64);
        let up = dynamic_overscan(400.0, 0.0, 20.0, 4, 64);
        assert_eq!(down, up);
    }

    #[test]
    fn dynamic_overscan_does_not_panic_on_a_non_positive_item_extent() {
        assert_eq!(dynamic_overscan(0.0, 400.0, 0.0, 4, 64), 4);
        assert_eq!(dynamic_overscan(0.0, 400.0, -5.0, 4, 64), 4);
    }
}
