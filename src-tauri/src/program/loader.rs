use std::{cell::RefCell, rc::Rc};

// No mutable references to require interior mutability instad of a mutible type
pub trait ProgressTracker {
    fn update_progress(&self, progress: f32);
}

type PRef = Rc<RefCell<_P>>;

struct _P {
    parent: Option<PRef>,
    progress: f32,
    weight: u64,
    children: Vec<PRef>,
    tracker: Option<Box<dyn ProgressTracker>>,
}

impl _P {
    pub fn update(&mut self) {
        match &self.tracker {
            Some(tracker) => {
                let mut total_weight = 0;
                let mut total_progress = 0.0;

                for c in &self.children {
                    let temp = c.borrow();

                    let partial_progress = temp.progress * temp.weight as f32;

                    total_progress += partial_progress;
                    total_weight += temp.weight;
                }

                tracker.update_progress(total_progress / total_weight as f32);
            }
            None => (),
        }
    }
}

pub struct Progress {
    stuff: PRef,
}

impl Progress {
    pub fn new(weight: u64) -> Progress {
        let stuff = _P {
            parent: None,
            progress: 0.0,
            weight,
            children: Vec::new(),
            tracker: None,
        };

        Progress {
            stuff: Rc::new(RefCell::new(stuff)),
        }
    }

    pub fn append(&mut self, p: &[&Progress]) {
        let mut t_s = self.stuff.borrow_mut();

        if t_s.children.len() == 0 && t_s.weight > 0 {
            // move stuff into a child
            let c = _P {
                parent: Some(self.stuff.clone()),
                progress: t_s.progress,
                weight: t_s.weight,
                children: Vec::new(),
                tracker: None,
            };
            t_s.children.push(Rc::new(RefCell::new(c)));
        }

        for pg in p {
            let mut t_c = pg.stuff.borrow_mut();
            t_s.weight = t_c.weight;

            t_s.children.push(pg.stuff.clone());
            t_c.parent = Some(self.stuff.clone());
        }
    }

    pub fn set_tracker<T: ProgressTracker + 'static>(&mut self, tracker: T) {
        self.stuff.borrow_mut().tracker = Some(Box::new(tracker));
    }

    pub fn set_progress(&mut self, progress: f32) {
        let mut slf = self.stuff.borrow_mut();
        if slf.children.len() > 0 {
            // Idk if this should panic or if something else should happen.
            todo!();
        }

        slf.progress = progress;

        match &slf.tracker {
            Some(t) => t.update_progress(slf.progress),
            None => (),
        }

        let p = match &slf.parent {
            Some(p) => Some(p.clone()),
            None => None,
        };

        drop(slf);

        match p {
            Some(p) => p.borrow_mut().update(),
            None => (),
        }
    }

    pub fn add_progress(&mut self, amount: f32) {
        let p = self.stuff.borrow().progress;
        self.set_progress(p + amount);
    }
}
