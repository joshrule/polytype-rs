use crate::{
    atype::{Schema, Ty, Type, TypeSchema, Variable},
    Name,
};
use fnv::FnvHashMap;
use itertools::Itertools;
use std::{borrow::Borrow, cell::RefCell, hash::Hash};
use typed_arena::Arena;

struct Interner<'a, K>(RefCell<FnvHashMap<&'a K, ()>>);

struct SliceInterner<'a, K>(RefCell<FnvHashMap<&'a [K], ()>>);

impl<'a, K: Eq + Hash> Interner<'a, K> {
    fn with_capacity(n: usize) -> Self {
        Interner(RefCell::new(FnvHashMap::with_capacity_and_hasher(
            n,
            Default::default(),
        )))
    }
    pub fn intern<Q>(&self, value: K, make: impl FnOnce(K) -> &'a K) -> &'a K
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut table = self.0.borrow_mut();
        // TODO: might not be super-efficient
        if let Some((k, _)) = table.get_key_value(&value) {
            &k
        } else {
            let k = make(value);
            table.insert(k, ());
            k
        }
    }
}

impl<'a, K> Default for Interner<'a, K> {
    fn default() -> Self {
        Interner(RefCell::new(FnvHashMap::default()))
    }
}

impl<'a, K: Eq + Hash> SliceInterner<'a, K> {
    pub fn with_capacity(n: usize) -> Self {
        SliceInterner(RefCell::new(FnvHashMap::with_capacity_and_hasher(
            n,
            Default::default(),
        )))
    }
    pub fn intern(&self, value: &[K], make: impl FnOnce(&[K]) -> &'a [K]) -> &'a [K] {
        let mut table = self.0.borrow_mut();
        // TODO: might not be super-efficient
        let opt: Option<(&&'a [K], _)> = table.get_key_value(value);
        if let Some((k, _)) = opt {
            *k
        } else {
            let k = make(value);
            table.insert(k, ());
            k
        }
    }
}

impl<'a, K> Default for SliceInterner<'a, K> {
    fn default() -> Self {
        SliceInterner(RefCell::new(FnvHashMap::default()))
    }
}

#[derive(Copy)]
pub struct TypeContext<'ctx, N: Name = &'static str> {
    ctx: &'ctx Context<'ctx, N>,
}

impl<'ctx, N: Name> PartialEq for TypeContext<'ctx, N> {
    fn eq(&self, other: &Self) -> bool {
        self.ctx as *const Context<'ctx, N> == other.ctx as *const Context<'ctx, N>
    }
}

impl<'ctx, N: Name> std::fmt::Debug for TypeContext<'ctx, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TypeContext<'ctx, N> {{ ctx: {:p} }}", self.ctx)
    }
}

impl<'ctx, N: Name> Eq for TypeContext<'ctx, N> {}

impl<'ctx, N: Name> Clone for TypeContext<'ctx, N> {
    fn clone(&self) -> Self {
        TypeContext { ctx: self.ctx }
    }
}

pub struct Context<'ctx, N: Name = &'static str> {
    schema_arena: &'ctx Arena<TypeSchema<'ctx, N>>,
    schema_map: Interner<'ctx, TypeSchema<'ctx, N>>,
    type_arena: &'ctx Arena<Type<'ctx, N>>,
    type_map: Interner<'ctx, Type<'ctx, N>>,
    name_arena: &'ctx Arena<N>,
    name_map: Interner<'ctx, N>,
    arg_arena: &'ctx Arena<Ty<'ctx, N>>,
    arg_map: SliceInterner<'ctx, Ty<'ctx, N>>,
}

pub fn with_ctx<F, R, M: Name>(n: usize, f: F) -> R
where
    F: FnOnce(TypeContext<'_, M>) -> R,
{
    let schemaa: Arena<TypeSchema<'_, M>> = Arena::with_capacity(n);
    let typea: Arena<Type<'_, M>> = Arena::with_capacity(n);
    let namea: Arena<M> = Arena::with_capacity(n);
    let tya: Arena<Ty<'_, M>> = Arena::with_capacity(n);
    let ctx = Context::with_capacity(&schemaa, &typea, &namea, &tya, n);
    let ctx: TypeContext<'_, M> = TypeContext::new(&ctx);
    f(ctx)
}

impl<'ctx, N: Name> TypeContext<'ctx, N> {
    pub fn new(ctx: &'ctx Context<'ctx, N>) -> Self {
        TypeContext { ctx }
    }
    pub fn intern_tvar(&self, v: Variable) -> Ty<'ctx, N> {
        let t = Type::Variable(v);
        self.ctx
            .type_map
            .intern(t, |t| self.ctx.type_arena.alloc(t))
    }
    pub(crate) fn intern_args(&self, args: &[Ty<'ctx, N>]) -> &'ctx [Ty<'ctx, N>] {
        self.ctx.arg_map.intern(args, |args| {
            self.ctx.arg_arena.alloc_extend(args.iter().copied())
        })
    }
    pub fn intern_tcon(&self, head: &'ctx N, args: &[Ty<'ctx, N>]) -> Ty<'ctx, N> {
        let body = self.intern_args(args);
        let t = Type::Constructed(head, body);
        self.ctx
            .type_map
            .intern(t, |t| self.ctx.type_arena.alloc(t))
    }
    pub fn arrow(&self, alpha: Ty<'ctx, N>, beta: Ty<'ctx, N>) -> Ty<'ctx, N> {
        let head = self.intern_name(N::arrow());
        self.intern_tcon(head, &[alpha, beta])
    }
    pub fn arrow_slice(&self, tps: &[Ty<'ctx, N>]) -> Ty<'ctx, N> {
        tps.iter()
            .rev()
            .copied()
            .fold1(|x, y| self.arrow(y, x))
            .unwrap_or_else(|| panic!("cannot create a type from nothing"))
    }
    pub fn intern_monotype(&self, inner: Ty<'ctx, N>) -> Schema<'ctx, N> {
        let t = TypeSchema::Monotype(inner);
        self.ctx
            .schema_map
            .intern(t, |t| self.ctx.schema_arena.alloc(t))
    }
    pub fn intern_polytype(&self, v: Variable, body: Schema<'ctx, N>) -> Schema<'ctx, N> {
        let t = TypeSchema::Polytype(v, body);
        self.ctx
            .schema_map
            .intern(t, |t| self.ctx.schema_arena.alloc(t))
    }
    pub fn intern_name(&self, n: N) -> &'ctx N {
        self.ctx
            .name_map
            .intern(n, |n| self.ctx.name_arena.alloc(n))
    }
}

impl<'ctx, N: Name> Context<'ctx, N> {
    pub fn new(
        schema_arena: &'ctx Arena<TypeSchema<'ctx, N>>,
        type_arena: &'ctx Arena<Type<'ctx, N>>,
        name_arena: &'ctx Arena<N>,
        arg_arena: &'ctx Arena<Ty<'ctx, N>>,
    ) -> Self {
        Context {
            schema_arena,
            schema_map: Interner::default(),
            type_arena,
            type_map: Interner::default(),
            name_arena,
            name_map: Interner::default(),
            arg_arena,
            arg_map: SliceInterner::default(),
        }
    }
    pub fn with_capacity(
        schema_arena: &'ctx Arena<TypeSchema<'ctx, N>>,
        type_arena: &'ctx Arena<Type<'ctx, N>>,
        name_arena: &'ctx Arena<N>,
        arg_arena: &'ctx Arena<Ty<'ctx, N>>,
        n: usize,
    ) -> Self {
        Context {
            schema_arena,
            schema_map: Interner::with_capacity(n),
            type_arena,
            type_map: Interner::with_capacity(n),
            name_arena,
            name_map: Interner::with_capacity(n),
            arg_arena,
            arg_map: SliceInterner::with_capacity(n),
        }
    }
}
