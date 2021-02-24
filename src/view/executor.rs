use crate::doc::Object;
use crate::query::Query;
use crate::value::Set;
use crate::value::fields::{Key, Path, Segment};

/// A data structure containing render context that can be "forked" and passed
/// to a child context.
///
/// This struct is helpful if you want recursively call [`Resource::to_object`] to render
/// a document's primary data and included resources.
///
/// Since the `Context` struct requires a mutable (unique) reference to a document's
/// included resources, only one context can be operated on at a time. In other words, if
/// you want to access a context, it cannot have any children in scope. Since you can
/// only operate on a single context at time, a recursive implementation of [included
/// resources] and [sparse field-sets] is much easier.
///
/// [`Resource::to_object`]: ../trait.Resource.html#tymethod.to_object
/// [included resources]: http://jsonapi.org/format/#fetching-includes
/// [sparse field-sets]: http://jsonapi.org/format/#fetching-sparse-fieldsets
#[derive(Debug)]
pub struct Executor<'v, CtxT> {
    context: &'v CtxT,
    incl: &'v mut Set<Object>,
    path: Path,
    query: Option<&'v Query>,
}

impl<'v, CtxT> Executor<'v, CtxT> {
    /// Creates a new, root context.
    ///
    /// This constructor can only be used when creating a root context. A child context
    /// can be created with the `fork` method.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::value::fields::ParseKeyError;
    /// #
    /// # fn example() -> Result<(), ParseKeyError> {
    /// use json_api::value::Set;
    /// use json_api::view::Executor;
    ///
    /// let mut included = Set::new();
    /// let mut ctx = Executor::new(&(), "posts".parse()?, None, &mut included);
    /// #
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// # example().unwrap();
    /// # }
    /// ```
    pub fn new(context: &'v CtxT, query: Option<&'v Query>, included: &'v mut Set<Object>) -> Self {
        Executor {
            context,
            query,
            incl: included,
            path: Path::new(),
        }
    }
    pub fn fields<'val>(&'val self, kind: &Key) -> crate::value::fields::Fields<'val> {
        crate::value::fields::Fields(self.query
            .and_then(|q| q.fields.get(kind)))
    }

    /// Returns true if the field name is present in the current context's
    /// field-set or the current context's field-set does not exist.
    pub fn field(&self, kind: &Key, name: &str) -> bool {
        self.query
            .and_then(|q| q.fields.get(kind))
            .map_or(true, |f| f.contains(name))
    }

    /// Creates a new child context from `self`.
    pub fn fork<NewCtxT>(&mut self, key: &Key) -> Executor<NewCtxT>
        where NewCtxT: FromContext<CtxT>
    {
        Executor {
            context: FromContext::from(self.context),
            incl: self.incl,
            path: self.path.join(key),
            query: self.query,
        }
    }

    /// Access the current context
    ///
    /// You usually provide the context when calling the top-level `execute`
    /// function, or using the context factory in the Iron integration.
    pub fn context(&self) -> &'v CtxT {
        self.context
    }

    /// Adds the `value` to the context's included resource set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn include(&mut self, value: Object) -> bool {
        self.incl.insert(value)
    }

    /// Returns `true` if the context is valid with respect to parent context(s).
    ///
    /// If there is no parent context (i.e the current context represents the primary
    /// data of the document), this will always return `false`.
    ///
    /// if there is a parent context and this function returns `false`, this context can
    /// should be ignored.
    /*
    pub fn included(&self) -> bool {
        self.query.map_or(false, |q| q.include.contains(&self.path))
    }
    */
    pub fn key_included(&self, key: &Key) -> bool {
        self.query.map_or(false, |q| q.include.contains(&self.path.join(key)))
    }

    pub fn included(&self) -> bool {
        self.query.map_or(false, |q| q.include.contains(&self.path))
    }
    /*
    pub fn one_converter<'k, 'ev, 'n, 'nv, R, F, V, Fut, NewCtxT>(&'ev mut self, key: &'k Key, converter: F) -> OneConverter<'k, 'ev, 'v, CtxT, R, F, V>
        where F: FnOnce(Executor<'nv, NewCtxT>, Option<V>) -> Fut,
            Fut: Future<Output=Result<R, ResolveError>>,
            NewCtxT: 'nv,
    {
        OneConverter {
            key, exec3: &mut self,
            value: None,
            phantom: PhantomData,
            converter: Some(converter),
        }
    }
    */
    /*
    pub fn one_collect(&self) -> Collect<Option<crate::doc::Identifier>> {
        Collect {
            value: None,
        }
    }

    pub fn many_collect<I>(&self) -> Collect<I>
        where I: Iterator<Item=crate::doc::Identifier>
    {
        Collect {
            value: None,
        }
    }
    */
}

/*
use std::marker::PhantomData;
use std::future::Future;
use super::ResolveError;
use crate::Resource;

pub struct Collect<V> {
    value: Option<V>,
}

impl<V> Collect<V> {
    pub async fn convert(&mut self, value: V) -> Result<(), ResolveError>
    {
        self.value = Some(value);
        Ok(())
    }

    pub fn value(&mut self) -> Option<V> {
        self.value.take()
    }
}

pub struct OneConverter<'nv, NewCtxT, R, F, V> {
    exec3: Option<Executor<'nv, NewCtxT>>,
    value: Option<R>,
    converter: Option<F>,
    phantom: PhantomData<V>,
}

impl<'nv, NewCtxT, R, F, V> OneConverter<'nv, NewCtxT, R, F, V>
{
    pub fn new<'n, 'k, 'ev, 'v, Fut, CtxT>(exec: &'ev mut Executor<'v, CtxT>, key: &'k Key, converter: F) -> OneConverter<'nv, NewCtxT, R, F, V>
        where F: FnOnce(Executor<'nv, NewCtxT>, Option<V>) -> Fut,
            Fut: Future<Output=Result<R, ResolveError>>,
            NewCtxT: FromContext<CtxT> + 'nv,
            V: Resource<Context=NewCtxT>,
            'v: 'nv,
            'ev: 'nv,
    {
        let kind = V::kind();
        let mut exec4 = exec.fork(kind, key);

        OneConverter {
            exec3: Some(exec4),
            value: None,
            phantom: PhantomData,
            converter: Some(converter),
        }
    }

    
    pub fn convert<'this, 'fut, Fut>(&'this mut self, value: Option<V>) -> impl Future<Output=Result<(), ResolveError>> + 'fut
    where F: FnOnce(Executor<'nv, NewCtxT>, Option<V>) -> Fut,
        Fut: Future<Output=Result<R, ResolveError>>,
          V: Resource<Context=NewCtxT>,
          'this: 'fut,
          'nv: 'fut,
          'fut: 'nv,
    {
        let converter = self.converter.take().unwrap();
        let exec3 = self.exec3.take().unwrap();
        async move {
            let c = (converter)(exec3, value).await?;
            self.value = Some(c);
            Ok(())
        }
    }

    pub fn value(&mut self) -> R {
        self.value.take().expect("value unassigned. convert not called")
    }
} 

pub struct ManyConverter<'ev, 'v, CtxT, R, F, V> {
    key: Key,
    executor: &'ev mut Executor<'v, CtxT>,
    value: Option<R>,
    converter: Option<F>,
    phantom: PhantomData<V>,
}

impl<'ev, 'v, CtxT, R, F, V> ManyConverter<'ev, 'v, CtxT, R, F, V>
{
    pub fn new<Fut, NewCtxT>(executor: &'ev mut Executor<'v, CtxT>, key: Key, converter: F) -> ManyConverter<'ev, 'v, CtxT, R, F, V>
        where F: FnOnce(&mut Executor<'v, NewCtxT>, &V) -> Fut,
              Fut: Future<Output=Result<R, ResolveError>>,
    {
        ManyConverter {
            key, executor,
            value: None,
            phantom: PhantomData,
            converter: Some(converter),
        }
    }

    pub async fn convert<'this, Fut, NewCtxT, VI>(&'this mut self, value: &V) -> Result<(), ResolveError>
    where F: FnOnce(&mut Executor<'v, NewCtxT>, &V) -> Fut,
        Fut: Future<Output=Result<R, ResolveError>>,
          V: Iterator<Item=VI>,
          VI: Resource<Context=NewCtxT>,
          NewCtxT: FromContext<CtxT> + 'ev,
          'this: 'v,
    {
        let kind = VI::kind();
        let mut executor = self.executor.fork(kind, &self.key);
        let c = (self.converter.take().unwrap())(&mut executor, value).await?;
        self.value = Some(c);
        Ok(())
    }

    pub fn value(&mut self) -> R {
        self.value.take().expect("value unassigned. convert not called")
    }
} 
*/

/// Conversion trait for context types
///
/// Used to support different context types for different parts of an
/// application. By making each `GraphQL` type only aware of as much
/// context as it needs to, isolation and robustness can be
/// improved. Implement this trait if you have contexts that can
/// generally be converted between each other.
///
/// The empty tuple `()` can be converted into from any context type,
/// making it suitable for `GraphQL` that don't need _any_ context to
/// work, e.g. scalars or enums.
pub trait FromContext<T> {
    /// Perform the conversion
    fn from(value: &T) -> &Self;
}

/// Marker trait for types that can act as context objects for `GraphQL` types.
pub trait Context {}

impl<'a, C: Context> Context for &'a C {}

static NULL_CONTEXT: () = ();

impl<T> FromContext<T> for () {
    fn from(_: &T) -> &Self {
        &NULL_CONTEXT
    }
}

impl<T> FromContext<T> for T
where
    T: Context,
{
    fn from(value: &T) -> &Self {
        value
    }
}