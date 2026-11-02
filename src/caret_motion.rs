//! Visual-only caret interpolation. Engine selection and hit testing stay exact.
use crate::selection::Rect;

const DURATION_US: i64 = 100_000;

#[derive(Default)]
pub struct CaretMotion {
    displayed: Option<Rect>,
    target: Option<Rect>,
    from: Option<Rect>,
    start: i64,
    blink_start: i64,
    visible: bool,
}

impl CaretMotion {
    pub fn set_visible(&mut self, visible: bool, now: i64) {
        if visible && !self.visible {
            self.restart_blink(now);
        }
        self.visible = visible;
    }
    pub fn restart_blink(&mut self, now: i64) {
        self.blink_start = now;
    }
    pub fn blinking(&self, now: i64, enabled: bool, timeout: i64) -> bool {
        self.visible && enabled && (timeout <= 0 || now.saturating_sub(self.blink_start) < timeout)
    }
    /// WordCraft's 100ms linear fade, with the native system's blink period.
    pub fn opacity(&self, now: i64, enabled: bool, period: i64, timeout: i64) -> f64 {
        if !self.visible {
            return 0.0;
        }
        if !self.blinking(now, enabled, timeout) {
            return 1.0;
        }
        let half = (period / 2).max(100_000);
        let since = now.saturating_sub(self.blink_start).max(0);
        if since < half {
            return 1.0;
        }
        let phase = since % (2 * half);
        let into = if phase < half { phase } else { phase - half };
        let done = (into as f64 / 100_000.0).clamp(0.0, 1.0);
        if phase < half { done } else { 1.0 - done }
    }
    pub fn position(&self) -> Option<&Rect> {
        self.displayed.as_ref()
    }
    pub fn is_moving(&self) -> bool {
        self.from.is_some()
    }
    pub fn finish(&mut self) {
        self.displayed = self.target.clone();
        self.from = None;
    }

    pub fn retarget(&mut self, target: Option<Rect>, now: i64, animate: bool) {
        self.advance(now);
        if self.target == target {
            if !animate {
                self.finish();
            }
            return;
        }
        // New documents, hidden/selected carets and page transitions snap.
        // Never draw a caret flying through page margins or unrelated pages.
        let same_page =
            matches!((&self.displayed, &target), (Some(a), Some(b)) if a.page == b.page);
        self.from = if animate && same_page {
            self.displayed.clone()
        } else {
            None
        };
        self.target = target;
        self.restart_blink(now);
        self.start = now;
        if !self.is_moving() {
            self.finish();
        }
    }

    pub fn advance(&mut self, now: i64) {
        let (Some(from), Some(to)) = (&self.from, &self.target) else {
            return;
        };
        let t = ((now - self.start) as f64 / DURATION_US as f64).clamp(0.0, 1.0);
        if t >= 1.0 {
            self.finish();
            return;
        }
        let t = 1.0 - (1.0 - t).powi(3);
        let lerp = |a, b| a + (b - a) * t;
        self.displayed = Some(Rect {
            page: to.page,
            x: lerp(from.x, to.x),
            y: lerp(from.y, to.y),
            width: lerp(from.width, to.width),
            height: lerp(from.height, to.height),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rect(page: usize, x: f64) -> Option<Rect> {
        Some(Rect {
            page,
            x,
            y: 20.0,
            width: 1.0,
            height: 16.0,
        })
    }
    #[test]
    fn rapid_typing_retargets_from_current_visual_position_and_finishes() {
        let mut m = CaretMotion::default();
        m.retarget(rect(0, 10.0), 0, true);
        assert!(!m.is_moving());
        m.retarget(rect(0, 30.0), 10, true);
        m.advance(50_010);
        let x = m.position().unwrap().x;
        assert!(x > 10.0 && x < 30.0);
        m.retarget(rect(0, 40.0), 50_010, true);
        assert_eq!(m.position().unwrap().x, x);
        // Repeated layout refreshes must not restart the animation.
        m.retarget(rect(0, 40.0), 70_010, true);
        m.advance(150_010);
        assert_eq!(m.position(), rect(0, 40.0).as_ref());
        assert!(!m.is_moving());
    }
    #[test]
    fn reduced_motion_hidden_caret_and_page_changes_snap() {
        let mut m = CaretMotion::default();
        m.retarget(rect(0, 10.0), 0, true);
        m.retarget(rect(0, 30.0), 1, true);
        m.retarget(rect(0, 30.0), 2, false);
        assert_eq!(m.position(), rect(0, 30.0).as_ref());
        assert!(!m.is_moving());
        m.retarget(rect(1, 40.0), 3, true);
        assert!(!m.is_moving());
        assert_eq!(m.position(), rect(1, 40.0).as_ref());
        m.retarget(None, 4, true);
        assert!(m.position().is_none());
        assert!(!m.is_moving());
    }
}

#[cfg(test)]
mod blink_tests {
    use super::*;
    #[test]
    fn reference_fade_restarts_and_disabled_or_timed_out_blink_is_static() {
        let mut m = CaretMotion::default();
        m.set_visible(true, 0);
        let alpha = |m: &CaretMotion, t| m.opacity(t, true, 1_060_000, 10_000_000);
        assert_eq!(alpha(&m, 529_999), 1.0);
        assert!((alpha(&m, 580_000) - 0.5).abs() < 0.001);
        assert_eq!(alpha(&m, 700_000), 0.0);
        assert!((alpha(&m, 1_110_000) - 0.5).abs() < 0.001);
        m.restart_blink(700_000);
        assert_eq!(alpha(&m, 700_000), 1.0);
        assert_eq!(m.opacity(1_400_000, false, 1_060_000, 0), 1.0);
        assert_eq!(m.opacity(20_000_000, true, 1_060_000, 10_000_000), 1.0);
        m.set_visible(false, 1_400_000);
        assert_eq!(alpha(&m, 1_400_000), 0.0);
        assert!(!m.blinking(1_400_000, true, 0));
    }
}
