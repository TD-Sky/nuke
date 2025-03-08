#[derive(Debug, Clone)]
pub struct SlotVec<T> {
    base: Vec<Option<T>>,
    len: usize,
}

impl<T> Default for SlotVec<T> {
    fn default() -> Self {
        Self {
            base: Default::default(),
            len: Default::default(),
        }
    }
}

impl<T> SlotVec<T> {
    #[allow(dead_code)]
    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, elt: T) {
        if let Some(v) = self.base.iter_mut().find(|elt| elt.is_none()) {
            *v = Some(elt);
        } else {
            self.base.push(Some(elt));
        }
        self.len += 1;
    }

    pub fn drain<F>(&mut self, f: F) -> impl Iterator<Item = T> + use<'_, F, T>
    where
        F: Fn(&T) -> bool,
    {
        let Self { base, len } = self;

        base.iter_mut()
            .filter_map(move |elt| elt.take_if(|v| f(v)).inspect(|_| *len -= 1))
    }
}
