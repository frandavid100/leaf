//! Ruler state is derived from public layout; edits retain the engine selection.
use serde_json::json;
use wordcraft_doc::props::{TabAlign, TabLeader, TabStop};
use wordcraft_engine::{Selection, Session, layout::Placed};
/// Pointer magnet in document points: half a centimetre, with a small hold band.
/// Numeric properties and imported values never pass through this function.
pub const HALF_CM: f32 = 72.0 / 2.54 / 2.0;
pub fn snap_drag(value: f32, previous: Option<f32>, min: f32, max: f32) -> f32 {
    let low = (min / HALF_CM).ceil() * HALF_CM;
    let high = (max / HALF_CM).floor() * HALF_CM;
    if low > high {
        return value.clamp(min, max);
    }
    let snapped = if let Some(old) = previous.filter(|old| (value - *old).abs() <= HALF_CM * 0.58) {
        old
    } else {
        (value / HALF_CM).round() * HALF_CM
    };
    snapped.clamp(low, high)
}
pub fn snap_marker(state: &State, marker: Marker, value: f32, previous: Option<f32>) -> f32 {
    match marker {
        Marker::Left => snap_drag(value, previous, -1584., state.width - 18.),
        Marker::First => {
            state.left
                + snap_drag(
                    value - state.left,
                    previous.map(|p| p - state.left),
                    -1584.,
                    1584.,
                )
        }
        Marker::Right => {
            state.width
                - snap_drag(
                    state.width - value,
                    previous.map(|p| state.width - p),
                    -1584.,
                    state.width - 18.,
                )
        }
        Marker::Tab(_) | Marker::NewTab => snap_drag(value, previous, 0., state.width.min(1584.)),
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub page: usize,
    pub page_width: f32,
    pub origin: f32,
    pub width: f32,
    pub rtl: bool,
    pub left: f32,
    pub first: f32,
    pub right: f32,
    pub tabs: Vec<TabStop>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    Left,
    First,
    Right,
    Tab(usize),
    NewTab,
}
impl State {
    pub fn distance(&self, m: Marker) -> f32 {
        match m {
            Marker::Left => self.left,
            Marker::First => self.left + self.first,
            Marker::Right => self.width - self.right,
            Marker::Tab(n) => self.tabs.get(n).map_or(0.0, |t| t.pos),
            Marker::NewTab => 72.0_f32.min(self.width),
        }
    }
    pub fn x(&self, distance: f32) -> f32 {
        self.origin
            + if self.rtl {
                self.width - distance
            } else {
                distance
            }
    }
    pub fn distance_at(&self, x: f32) -> f32 {
        if self.rtl {
            self.origin + self.width - x
        } else {
            x - self.origin
        }
    }
}
#[derive(Clone, Debug)]
pub struct Lease {
    id: u64,
    rev: u64,
    selection: Selection,
}
impl Lease {
    pub fn capture(s: &Session) -> Self {
        Self {
            id: s.document_id(),
            rev: s.rev(),
            selection: s.sel.clone(),
        }
    }
    pub fn valid(&self, s: &Session) -> bool {
        self.id == s.document_id() && self.rev == s.rev() && self.selection == s.sel
    }
}
pub fn state(s: &mut Session) -> Option<State> {
    let pos = s.sel.focus.clone();
    let props = s.doc.styles.resolve_para(&s.doc.para_at(&pos)?.props);
    let layout = s.layout();
    let caret = layout.caret_on(&pos, s.page_hint)?;
    let page = layout.pages.get(caret.page)?;
    let mut col_x = None;
    for item in page.items.iter().chain(&page.header).chain(&page.footer) {
        if let Placed::Lines {
            story,
            path,
            para,
            l0,
            l1,
            x,
            y,
            turn,
        } = item
        {
            if *turn == wordcraft_doc::props::TextDirection::Horizontal
                && *story == pos.story
                && *path == pos.path
                && para.lines[*l0..*l1]
                    .iter()
                    .any(|l| l.start <= pos.off && pos.off <= l.stop)
                && caret.top >= *y - 0.5
                && caret.top < *y + para.lines[*l0..*l1].iter().map(|l| l.height).sum::<f32>() + 0.5
            {
                col_x = Some(*x);
                break;
            }
        }
    }
    let x = col_x?;
    let width = if let Some((table, row, cell)) = pos.path.cell() {
        let rect = page.items.iter().find_map(|p| match p {
            Placed::Cell {
                rect,
                table: t,
                row: r,
                cell: c,
                story,
                ..
            } if *t == table && *r == row && *c == cell && *story == pos.story => Some(rect),
            _ => None,
        })?;
        let margins = wordcraft_engine::layout::cell_margins(
            &s.doc,
            s.doc.table(pos.story, &table)?,
            row,
            cell,
        );
        (rect.w - margins[1].max(0.0) - margins[3].max(0.0)).max(18.0)
    } else if pos.story == wordcraft_doc::StoryRef::Body {
        let sections = s.doc.sections();
        let block = *pos.path.0.first()? as usize;
        let sect = sections.get(s.doc.section_index_of(block))?.1;
        sect.column_boxes()
            .into_iter()
            .min_by(|a, b| {
                (page.body.x + a.0 - x)
                    .abs()
                    .total_cmp(&(page.body.x + b.0 - x).abs())
            })
            .map(|c| c.1)?
    } else {
        page.body.w
    };
    Some(State {
        page: caret.page,
        page_width: page.w,
        origin: x,
        width,
        rtl: props.bidi,
        left: props.indent_left,
        first: props.indent_first,
        right: props.indent_right,
        tabs: props.tabs,
    })
}
pub fn edit(
    s: &mut Session,
    lease: &Lease,
    state: &State,
    marker: Marker,
    distance: f32,
    drop: bool,
) -> Result<bool, String> {
    if !lease.valid(s) {
        return Err(wordcraft_gnome::t!(
            "The document has changed; start the ruler gesture again."
        )
        .into());
    }
    if !distance.is_finite() {
        return Ok(false);
    }
    let (command, args) = match marker {
        Marker::Left => {
            let value = distance.clamp(-1584.0, state.width - 18.0);
            if (value - state.left).abs() < 0.0001 {
                return Ok(false);
            }
            ("para.indents", json!({"left":value}))
        }
        Marker::First => {
            let value = (distance - state.left).clamp(-1584.0, 1584.0);
            if (value - state.first).abs() < 0.0001 {
                return Ok(false);
            }
            ("para.indents", json!({"firstLine":value}))
        }
        Marker::Right => {
            let value = (state.width - distance).clamp(-1584.0, state.width - 18.0);
            if (value - state.right).abs() < 0.0001 {
                return Ok(false);
            }
            ("para.indents", json!({"right":value}))
        }
        Marker::Tab(n) => {
            let old = *state.tabs.get(n).ok_or("Missing tab stop")?;
            let at = distance.clamp(0.0, state.width).min(1584.0);
            if drop {
                ("para.tabs", json!({"clear":[old.pos]}))
            } else {
                if (at - old.pos).abs() < wordcraft_doc::props::TAB_POS_EPS {
                    return Ok(false);
                }
                let new = TabStop { pos: at, ..old };
                ("para.tabs", json!({"clear":[old.pos],"set":[new]}))
            }
        }
        Marker::NewTab => {
            if drop {
                return Ok(false);
            }
            let stop = TabStop {
                pos: distance.clamp(0.0, state.width).min(1584.0),
                align: TabAlign::Left,
                leader: TabLeader::None,
            };
            ("para.tabs", json!({"set":[stop]}))
        }
    };
    let before = s.rev();
    s.run(command, &args).map_err(|e| e.to_string())?;
    Ok(s.rev() != before)
}
