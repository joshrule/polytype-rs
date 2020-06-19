use crate::{
    atype::{Ty, TypeContext, Variable},
    Name,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Snapshot(usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Substitution<'ctx, N: Name = &'static str> {
    pub(crate) ctx: TypeContext<'ctx, N>,
    pub(crate) sub: Vec<(Variable, Ty<'ctx, N>)>,
}

pub struct SubIter<'a, 'ctx, N: Name = &'static str> {
    it: std::slice::Iter<'a, (Variable, Ty<'ctx, N>)>,
}
pub struct SubIterMut<'a, 'ctx, N: Name = &'static str> {
    it: std::slice::IterMut<'a, (Variable, Ty<'ctx, N>)>,
}

impl<'ctx, N: Name> Substitution<'ctx, N> {
    pub fn with_capacity(ctx: TypeContext<'ctx, N>, n: usize) -> Self {
        Substitution {
            ctx,
            sub: Vec::with_capacity(n),
        }
    }
    pub fn get(&self, q: Variable) -> Option<Ty<'ctx, N>> {
        self.sub.iter().find(|(k, _)| *k == q).map(|(_, v)| *v)
    }
    pub fn add(&mut self, k: Variable, v: Ty<'ctx, N>) -> bool {
        if self.sub.iter().any(|(j, _)| k == *j) {
            false
        } else {
            self.sub.push((k, v));
            true
        }
    }
    /// The `Substitution` as a slice.
    pub fn as_slice(&self) -> &[(Variable, Ty<'ctx, N>)] {
        &self.sub
    }
    /// An Iterator over the `Substitution`.
    pub fn iter<'a>(&'a self) -> SubIter<'a, 'ctx, N> {
        SubIter {
            it: self.sub.iter(),
        }
    }
    /// A mutable Iterator over the `Substitution`.
    pub fn iter_mut<'a>(&'a mut self) -> SubIterMut<'a, 'ctx, N> {
        SubIterMut {
            it: self.sub.iter_mut(),
        }
    }
    /// The number of constraints in the `Substitution`.
    pub fn len(&self) -> usize {
        self.sub.len()
    }
    /// `true` if the `Substitution` has any constraints, else `false`.
    pub fn is_empty(&self) -> bool {
        self.sub.is_empty()
    }
    /// Clears the substitution managed by the context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use polytype::atype::{with_ctx, Substitution, TypeContext, Variable};
    /// # use polytype::Source;
    /// with_ctx(10, |ctx: TypeContext<'_>| {
    ///     let mut src = Source::default();
    ///     let mut sub = Substitution::with_capacity(ctx, 1);
    ///
    ///     let clean = sub.clone();
    ///
    ///     let t_var = Variable(src.fresh());
    ///     let t1 = ctx.intern_tvar(Variable(src.fresh()));
    ///     sub.add(t_var, t1);
    ///
    ///     let dirty = sub.clone();
    ///
    ///     sub.clean();
    ///
    ///     assert_eq!(clean, sub);
    ///     assert_ne!(clean, dirty);
    /// })
    /// ```
    pub fn clean(&mut self) {
        self.sub.clear();
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot(self.len())
    }
    /// Removes all but `n` substitutions added to the `Context`.
    pub fn rollback(&mut self, Snapshot(n): Snapshot) {
        self.sub.truncate(n)
    }
}

impl<'a, 'ctx, N: Name> Iterator for SubIter<'a, 'ctx, N> {
    type Item = &'a (Variable, Ty<'ctx, N>);
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}

impl<'a, 'ctx, N: Name> Iterator for SubIterMut<'a, 'ctx, N> {
    type Item = &'a mut (Variable, Ty<'ctx, N>);
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}
