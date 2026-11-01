use super::*;

// The actual visible canvas implements GtkAccessibleText. This is a derived,
// read-only view of WordCraft, not a hidden GtkTextView or an editing surrogate.
pub(super) mod document_canvas {
    use super::*;
    use gtk::subclass::prelude::*;
    use wordcraft_doc::StoryRef;
    use wordcraft_gnome::accessible_text::TextIndex;

    #[derive(Default)]
    struct State {
        identity: Option<(u64, u64, StoryRef)>,
        text: TextIndex,
        caret: u32,
        selection: Option<(u32, u32)>,
        readonly: bool,
    }
    mod imp {
        use super::*;
        #[derive(Default)]
        pub struct DocumentCanvas {
            pub(super) state: RefCell<State>,
            pub(super) ui: RefCell<std::rc::Weak<Ui>>,
        }
        #[glib::object_subclass]
        impl ObjectSubclass for DocumentCanvas {
            const NAME: &'static str = "LeafDocumentCanvas";
            type Type = super::DocumentCanvas;
            type ParentType = gtk::Fixed;
            type Interfaces = (gtk::AccessibleText,);
            fn class_init(class: &mut Self::Class) {
                class.set_accessible_role(gtk::AccessibleRole::TextBox);
            }
        }
        impl ObjectImpl for DocumentCanvas {}
        impl WidgetImpl for DocumentCanvas {}
        impl FixedImpl for DocumentCanvas {}
        impl AccessibleTextImpl for DocumentCanvas {
            fn contents(&self, start: u32, end: u32) -> Option<glib::Bytes> {
                let state = self.state.borrow();
                let text = state.text.contents(start, end)?;
                // GtkAccessibleText's UTF-8 GBytes contract includes a final NUL.
                let mut bytes = Vec::with_capacity(text.len() + 1);
                bytes.extend_from_slice(text.as_bytes());
                bytes.push(0);
                Some(glib::Bytes::from_owned(bytes))
            }
            fn contents_at(
                &self,
                offset: u32,
                granularity: gtk::AccessibleTextGranularity,
            ) -> Option<(u32, u32, glib::Bytes)> {
                let range = {
                    let state = self.state.borrow();
                    match granularity {
                        gtk::AccessibleTextGranularity::Character => state.text.character(offset),
                        gtk::AccessibleTextGranularity::Word => state.text.word(offset),
                        gtk::AccessibleTextGranularity::Sentence => state.text.sentence(offset),
                        gtk::AccessibleTextGranularity::Paragraph => state.text.paragraph(offset),
                        gtk::AccessibleTextGranularity::Line => {
                            let ui = self.ui.borrow().upgrade()?;
                            let mut current = ui.current.try_borrow_mut().ok()?;
                            state.text.line(&mut current.as_mut()?.session, offset)
                        }
                        _ => None,
                    }
                }?;
                Some((range.0, range.1, self.contents(range.0, range.1)?))
            }
            fn caret_position(&self) -> u32 {
                self.state.borrow().caret
            }
            fn selection(&self) -> Vec<gtk::AccessibleTextRange> {
                self.state
                    .borrow()
                    .selection
                    .map(|(a, b)| gtk::AccessibleTextRange::new(a as usize, (b - a) as usize))
                    .into_iter()
                    .collect()
            }
            fn attributes(
                &self,
                _offset: u32,
            ) -> Vec<(gtk::AccessibleTextRange, glib::GString, glib::GString)> {
                Vec::new()
            }
            fn default_attributes(&self) -> Vec<(glib::GString, glib::GString)> {
                vec![(
                    "editable".into(),
                    if self.state.borrow().readonly {
                        "false"
                    } else {
                        "true"
                    }
                    .into(),
                )]
            }
        }
    }
    glib::wrapper! {
        pub struct DocumentCanvas(ObjectSubclass<imp::DocumentCanvas>)
        @extends gtk::Fixed,gtk::Widget,
        @implements gtk::Accessible,gtk::AccessibleText,gtk::Buildable,gtk::ConstraintTarget;
    }
    impl DocumentCanvas {
        pub fn new() -> Self {
            glib::Object::new()
        }
        pub(in crate::gnome) fn bind(&self, ui: &Rc<Ui>) {
            self.imp().ui.replace(Rc::downgrade(ui));
        }
        pub(in crate::gnome) fn reset(&self) {
            let len = self.imp().state.borrow().text.len();
            if len > 0 {
                self.update_contents(gtk::AccessibleTextContentChange::Remove, 0, len);
            }
            self.imp().state.replace(State::default());
            self.update_caret_position();
            self.update_selection_bound();
        }
        pub(in crate::gnome) fn sync(&self, ui: &Rc<Ui>) {
            let (identity, next, caret, selection, readonly) = {
                let Ok(current) = ui.current.try_borrow() else {
                    return;
                };
                let Some(doc) = current.as_ref() else {
                    return;
                };
                let s = &doc.session;
                let identity = (s.document_id(), s.rev(), s.sel.focus.story);
                let state = self.imp().state.borrow();
                let next = (state.identity != Some(identity))
                    .then(|| TextIndex::new(&s.doc, s.sel.focus.story));
                let text = next.as_ref().unwrap_or(&state.text);
                let caret = text.offset(&s.sel.focus).unwrap_or(0);
                let selection = text.selection(s);
                let readonly = s
                    .doc
                    .settings
                    .protection
                    .as_deref()
                    .is_some_and(|p| matches!(p, "readOnly" | "comments" | "forms"));
                (identity, next, caret, selection, readonly)
            };
            let change = next
                .as_ref()
                .and_then(|n| self.imp().state.borrow().text.change(n));
            // No RefCell borrow is held while GTK emits synchronous notifications:
            // clients must be able to query the OLD deletion and NEW insertion.
            if let Some(c) = change.filter(|c| c.old_end > c.start) {
                self.update_contents(gtk::AccessibleTextContentChange::Remove, c.start, c.old_end);
            }
            let (caret_changed, selection_changed, readonly_changed) = {
                let mut state = self.imp().state.borrow_mut();
                let changed = (
                    state.caret != caret,
                    state.selection != selection,
                    state.readonly != readonly,
                );
                if let Some(next) = next {
                    state.text = next;
                }
                state.identity = Some(identity);
                state.caret = caret;
                state.selection = selection;
                state.readonly = readonly;
                changed
            };
            if let Some(c) = change.filter(|c| c.new_end > c.start) {
                self.update_contents(gtk::AccessibleTextContentChange::Insert, c.start, c.new_end);
            }
            if caret_changed {
                self.update_caret_position();
            }
            if selection_changed {
                self.update_selection_bound();
            }
            if readonly_changed {
                self.update_property(&[gtk::accessible::Property::ReadOnly(readonly)]);
            }
        }
    }
}
pub(super) use document_canvas::DocumentCanvas;
