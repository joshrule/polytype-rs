use crate::{
    atype::{Schema, Ty, Type, TypeContext, TypeSchema, Variable},
    Name,
};

/// A way to generate fresh `Variable`s.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Source(pub(crate) usize);

/// Allow types to be reified for use under a different `Source`.
pub struct SourceChange {
    pub(crate) delta: usize,
    pub(crate) sacreds: Vec<Variable>,
}

impl Source {
    /// Get a fresh bare [`Variable`].
    ///
    /// [`Variable`]: type.Variable.html
    pub fn fresh(&mut self) -> usize {
        let var = self.0;
        self.0 += 1;
        var
    }
    pub fn merge(&mut self, other: Self, sacreds: Vec<Variable>) -> SourceChange {
        let delta = self.0;
        // this is intentionally wasting variable space when there are sacreds:
        self.0 += other.0;
        SourceChange { delta, sacreds }
    }
}

impl Default for Source {
    fn default() -> Self {
        Source(0)
    }
}

impl SourceChange {
    /// Reify a `Type` for use under a new [`Source`].
    ///
    /// [`Source`]: struct.Source.html
    pub fn reify_type<'ctx, N: Name>(
        &self,
        tp: Ty<'ctx, N>,
        ctx: &TypeContext<'ctx, N>,
    ) -> Ty<'ctx, N> {
        match *tp {
            Type::Constructed(head, args) => {
                let mut new_args = Vec::with_capacity(args.len());
                for arg in args {
                    new_args.push(self.reify_type(arg, ctx));
                }
                let new_args = ctx.intern_args(&new_args);
                ctx.intern_tcon(head, new_args)
            }
            Type::Variable(n) if self.sacreds.contains(&n) => tp,
            Type::Variable(n) => ctx.intern_tvar(Variable(n.0 + self.delta)),
        }
    }
    /// Reify a `TypeSchema` for use under a new [`Source`].
    ///
    /// [`Source`]: struct.Source.html
    pub fn reify_typeschema<'ctx, N: Name>(
        &self,
        schema: &Schema<'ctx, N>,
        ctx: &TypeContext<'ctx, N>,
    ) -> Schema<'ctx, N> {
        match *schema {
            TypeSchema::Monotype(inner) => ctx.intern_monotype(self.reify_type(inner, ctx)),
            TypeSchema::Polytype(variable, body) => {
                let t_var = Variable(variable.0 + self.delta);
                let new_body = self.reify_typeschema(body, ctx);
                ctx.intern_polytype(t_var, new_body)
            }
        }
    }
}
