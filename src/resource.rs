use std::mem;
use async_trait::async_trait;

use crate::doc::{Data, Document, Identifier, Object};
use crate::query::Query;
use crate::value::Set;
use crate::value::fields::Key;
use crate::view::{Executor, Resolver, ResolveError};

pub trait StaticKind {
    fn static_kind() -> Key;
}

impl<'t, T:StaticKind> StaticKind for &'t T {
    fn static_kind() -> Key {
        T::static_kind()
    }
}

pub trait StaticResource: Resource + StaticKind {}
impl<'t, T:StaticResource> StaticResource for &'t T {}

/// A trait indicating that the given type can be represented as a resource.
///
/// Implementing this trait manually is not recommended. The [`resource!`] macro provides
/// a friendly DSL that implements trait with some additional functionality.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate json_api;
///
/// struct Post(u64);
///
/// resource!(Post, |&self, _ctx: &()| {
///     kind "posts";
///     id self.0;
/// });
/// #
/// # fn main() {}
/// ```
///
/// [`resource!`]: ./macro.resource.html
#[async_trait]
pub trait Resource {
    type Context;

    /// Returns a key containing the type of resource.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate json_api;
    /// #
    /// # struct Post(u64);
    /// #
    /// # resource!(Post, |&self, _ctx: &()| {
    /// #     kind "posts";
    /// #     id self.0;
    /// # });
    /// #
    /// # async fn example() {
    /// use json_api::Resource;
    ///
    /// let kind = Post::kind().await;
    /// assert_eq!(kind, "posts");
    /// # }
    /// # fn main() {}
    /// ```
    async fn kind(&self) -> Key;

    /// Returns a given resource's id as a string.
    ///
    /// # Example
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate json_api;
    /// #
    /// # struct Post(u64);
    /// #
    /// # resource!(Post, |&self, _ctx: &()| {
    /// #     kind "posts";
    /// #     id self.0;
    /// # });
    /// #
    /// # async fn example() {
    /// use json_api::Resource;
    ///
    /// let post = Post(25);
    /// assert_eq!(post.id().await, "25");
    /// # }
    /// # fn main() {}
    /// ```
    async fn id(&self) -> String;

    /// Renders a given resource as an identifier object.
    ///
    ///
    /// Calling this function directly is not recommended. It is much more ergonomic to
    /// use the [`json_api::to_doc`] function.
    ///
    /// [`json_api::to_doc`]: ./fn.to_doc.html
    async fn to_ident<'this, 'e, 'ev>(&'this self, executor: &'e mut Executor<'ev, Self::Context>) -> Result<Identifier, ResolveError>;

    /// Renders a given resource as a resource object.
    ///
    /// Calling this function directly is not recommended. It is much more ergonomic to
    /// use the [`json_api::to_doc`] function.
    ///
    /// [`json_api::to_doc`]: ./fn.to_doc.html
    async fn to_object<'this, 'e, 'ev>(&'this self, executor: &'e mut Executor<'ev, Self::Context>) -> Result<Object, ResolveError>;
}


impl<'a, T: Resource> Resource for &'a T {
    type Context = T::Context;

    fn kind<'this, 'async_trait>(&'this self) -> core::pin::Pin<Box<dyn core::future::Future<Output = Key> + Send + 'async_trait>>
        where 'this: 'async_trait,
    {
        (*self).kind()
    }
    fn id<'this, 'async_trait>(&'this self) -> core::pin::Pin<Box<dyn core::future::Future<Output = String> + Send + 'async_trait>>
        where 'this: 'async_trait,
    {
        (*self).id()
    }
    fn to_ident<'this, 'e, 'ev, 'async_trait>(
        &'this self,
        executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Identifier, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              'a: 'async_trait,
              Self: 'async_trait,
    {
        (*self).to_ident(executor)
    }

    fn to_object<'this, 'e, 'ev, 'async_trait>(
        &'this self,
        executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Object, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              'a: 'async_trait,
              Self: 'async_trait,
    {
        (*self).to_object(executor)
    }
}

use std::marker::PhantomData;
use futures_util::future::ready;

pub struct ObjectWrap<CtxT> {
    obj: Option<Object>,
    phantom: PhantomData<CtxT>,
}

impl<CtxT> ObjectWrap<CtxT> {
    pub fn new<'v>(obj: Object, _executor: &Executor<'v, CtxT>) -> ObjectWrap<CtxT> {
        ObjectWrap {
            obj: Some(obj),
            phantom: PhantomData,
        }
    }
}

pub fn object_wrap<'v, CtxT>(obj: Object, executor: &Executor<'v, CtxT>) -> impl InternalResolve<Context=CtxT> {
    ObjectWrap::new(obj, executor)
}

pub trait InternalResolve {
    type Context;

    fn resolve_ident<'this, 'e, 'ev, 'async_trait>(
        &'this self,
        _executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Identifier, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              Self: 'async_trait;
    fn resolve_object<'this, 'e, 'ev, 'async_trait>(
        &'this mut self,
        executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Object, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              Self: 'async_trait;
}

impl<CtxT> InternalResolve for ObjectWrap<CtxT> {
    type Context = CtxT;

    fn resolve_ident<'this, 'e, 'ev, 'async_trait>(
        &'this self,
        _executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Identifier, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              Self: 'async_trait,
    {
        let obj = self.obj.as_ref().unwrap();
        let id = Identifier::new(obj.kind.clone(), obj.id.clone());
        Box::pin(ready(Ok(id)))
    }

    fn resolve_object<'this, 'e, 'ev, 'async_trait>(
        &'this mut self,
        _executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Object, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              Self: 'async_trait,
    {
        let obj = self.obj.take().unwrap();
        Box::pin(ready(Ok(obj)))
    }
}
impl<A: Resource> InternalResolve for A {
    type Context = A::Context;

    fn resolve_ident<'this, 'e, 'ev, 'async_trait>(
        &'this self,
        executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Identifier, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              Self: 'async_trait,
    {
        self.to_ident(executor)
    }

    fn resolve_object<'this, 'e, 'ev, 'async_trait>(
        &'this mut self,
        executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Object, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              Self: 'async_trait,
    {
        self.to_object(executor)
    }
}


impl<'a, A, B, Ctx> Resource for futures_util::future::Either<A, B>
    where A: Resource<Context=Ctx> + Sync + Send, 
          B: Resource<Context=Ctx> + Sync + Send,
          Ctx: Sync,
{
    type Context = A::Context;

    fn kind<'this, 'async_trait>(&'this self) -> core::pin::Pin<Box<dyn core::future::Future<Output = Key> + Send + 'async_trait>>
        where 'this: 'async_trait,
    {
        use futures_util::future::Either;
        Box::pin(async move {
            match self {
                Either::Left(a) => a.kind().await,
                Either::Right(b) => b.kind().await,
            }
        })
    }
    fn id<'this, 'async_trait>(&'this self) -> core::pin::Pin<Box<dyn core::future::Future<Output = String> + Send + 'async_trait>>
        where 'this: 'async_trait,
    {
        use futures_util::future::Either;
        Box::pin(async move {
            match self {
                Either::Left(a) => a.id().await,
                Either::Right(b) => b.id().await,
            }
        })
    }
    fn to_ident<'this, 'e, 'ev, 'async_trait>(
        &'this self,
        executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Identifier, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              Self: 'async_trait,
    {
        use futures_util::future::Either;
        Box::pin(async move {
            match self {
                Either::Left(a) => a.to_ident(executor).await,
                Either::Right(b) => b.to_ident(executor).await,
            }
        })
    }

    fn to_object<'this, 'e, 'ev, 'async_trait>(
        &'this self,
        executor: &'e mut Executor<'ev, Self::Context>,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<Object, ResolveError>> + Send + 'async_trait>>
        where 'this: 'async_trait,
              'e: 'async_trait,
              'ev: 'async_trait,
              Self: 'async_trait,
    {
        use futures_util::future::Either;
        Box::pin(async move {
            match self {
                Either::Left(a) => a.to_object(executor).await,
                Either::Right(b) => b.to_object(executor).await,
            }
        })
    }
}

#[async_trait]
impl<'a, T> Resolver<Identifier> for &'a T
    where T: Resource + Send + Sync,
          <T as Resource>::Context: Send + Sync,
{
    type Context = T::Context;
    async fn resolve(self, query: Option<&Query>, ctx: &Self::Context) -> Result<Document<Identifier>, ResolveError> {
        let mut incl = Set::new();
        let mut executor = Executor::new(ctx, query, &mut incl);

        self.to_ident(&mut executor).await?.resolve(query, &()).await
    }
}

#[async_trait]
impl<'a, T> Resolver<Identifier> for &'a [T]
    where T: Resource + Send + Sync,
          <T as Resource>::Context: Send + Sync,
{
    type Context = T::Context;
    async fn resolve(self, query: Option<&Query>, ctx: &Self::Context) -> Result<Document<Identifier>, ResolveError> {
        let mut incl = Set::new();
        let mut executor = Executor::new(ctx, query, &mut incl);

        let mut ret = Vec::with_capacity(self.len());
        for item in self.into_iter() {
            ret.push(item.to_ident(&mut executor).await?);
        }
        ret.resolve(query, &()).await
    }
}


#[async_trait]
impl<'a, T> Resolver<Object> for &'a T
    where T: Resource + Send + Sync,
          <T as Resource>::Context: Sync + Send,
{
    type Context = T::Context;
    async fn resolve(self, query: Option<&Query>, ctx: &Self::Context) -> Result<Document<Object>, ResolveError> {
        let mut incl = Set::new();
        let (data, links, meta) = {
            let mut executor = Executor::new(ctx, query, &mut incl);
            let mut obj = self.to_object(&mut executor).await?;
            let links = mem::replace(&mut obj.links, Default::default());
            let meta = mem::replace(&mut obj.meta, Default::default());

            (obj.into(), links, meta)
        };

        Ok(Document::Ok {
            data,
            links,
            meta,
            included: incl,
            jsonapi: Default::default(),
        })
    }
}

#[async_trait]
impl<'a, T> Resolver<Object> for &'a [T]
    where T: Resource + Send + Sync,
          <T as Resource>::Context: Send + Sync,
{
    type Context = T::Context;
    async fn resolve(self, query: Option<&Query>, ctx: &Self::Context) -> Result<Document<Object>, ResolveError> {
        let mut incl = Set::new();
        let mut data = Vec::with_capacity(self.len());

        {
            let mut executor = Executor::new(ctx, query, &mut incl);

            for item in self {
                data.push(item.to_object(&mut executor).await?);
            }
        }

        Ok(Document::Ok {
            data: Data::Collection(data),
            links: Default::default(),
            meta: Default::default(),
            included: incl,
            jsonapi: Default::default(),
        })
    }
}

/// A DSL for implementing the `Resource` trait.
///
/// # Examples
///
/// The `resource!` macro is both concise and flexible. Many of the keywords are
/// overloaded to provide a higher level of customization when necessary.
///
/// Here is a simple example that simply defines the resources id, kind, attributes, and
/// relationships.
///
/// ```
/// #[macro_use]
/// extern crate json_api;
///
/// struct Post {
///     id: u64,
///     body: String,
///     title: String,
///     author: Option<User>,
///     comments: Vec<Comment>,
/// }
///
/// resource!(Post, |&self, _ctx: &()| {
///     // Define the id.
///     id self.id;
///
///     // Define the resource "type"
///     kind "posts";
///
///     // Define attributes with a comma seperated list of field names.
///     attrs body, title;
///
///     // Define relationships with a comma seperated list of field names.
///     has_one author;
///     has_many comments;
/// });
/// #
/// # struct User;
/// #
/// # resource!(User, |&self, _ctx: &()| {
/// #     kind "users";
/// #     id String::new();
/// # });
/// #
/// # struct Comment;
/// #
/// # resource!(Comment, |&self, _ctx: &()| {
/// #     kind "comments";
/// #     id String::new();
/// # });
/// #
/// # fn main() {}
/// ```
///
/// Now let's take a look at how we can use the same DSL to get a higher level
/// customization.
///
/// ```
/// #[macro_use]
/// extern crate json_api;
///
/// struct Post {
///     id: u64,
///     body: String,
///     title: String,
///     author: Option<User>,
///     comments: Vec<Comment>,
/// }
///
/// resource!(Post, |&self, _ctx: &()| {
///     kind "articles";
///     id self.id;
///
///     attrs body, title;
///
///     // Define a virtual attribute with an expression
///     attr "preview", {
///         self.body
///             .chars()
///             .take(140)
///             .collect::<String>()
///     }
///
///     // Define a relationship with granular detail
///     has_one "author", {
///         // Data for has one should be Option<&T> where T: Resource
///         data self.author.as_ref();
///
///         // Define relationship links
///         link "self", format!("/articles/{}/relationships/author", self.id);
///         link "related", format!("/articles/{}/author", self.id);
///
///         // Define arbitrary meta members with a block expression
///         meta "read-only", true;
///     }
///
///     // Define a relationship with granular detail
///     has_many "comments", {
///         // Data for has one should be an Iterator<Item = &T> where T: Resource
///         data self.comments.iter();
///
///         // Define relationship links
///         link "self", format!("/articles/{}/relationships/comments", self.id);
///         link "related", format!("/articles/{}/comments", self.id);
///
///         // Define arbitrary meta members with a block expression
///         meta "total", {
///             self.comments.len()
///         }
///     }
///
///     // You can also define links with granular details as well
///     link "self", {
///         href format!("/articles/{}", self.id);
///     }
///
///     // Define arbitrary meta members an expression
///     meta "copyright", self.author.as_ref().map(|user| {
///         format!("© 2017 {}", user.full_name())
///     });
/// });
/// #
/// # struct User;
/// #
/// # impl User {
/// #     fn full_name(&self) -> String {
/// #         String::new()
/// #     }
/// # }
/// #
/// # resource!(User, |&self, _ctx: &()| {
/// #     kind "users";
/// #     id String::new();
/// # });
/// #
/// # struct Comment;
/// #
/// # resource!(Comment, |&self, _ctx: &()| {
/// #     kind "comments";
/// #     id String::new();
/// # });
/// #
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! resource {
    ($target:ident, |&$this:ident| { $($rest:tt)* }) => {
        $crate::resource!($target, |&$this, _ctx:&()| {
            $($rest)*
        });
    };
    ($target:ident, |&$this:ident, $ctx:ident:&$context:ty| { $($rest:tt)* }) => {
        impl $crate::StaticResource for $target {}
        impl $crate::StaticKind for $target {
            fn static_kind() -> $crate::value::Key {
                let raw = $crate::extract_resource_kind!({ $($rest)* }).to_owned();
                $crate::value::Key::from_raw(raw)
            }
        }

        impl $crate::Resource for $target {
            $crate::resource_body!(|&$this, $ctx:&$context| {
                $($rest)*
            });
        }
    };
    ($target:ident<$($lf:lifetime,)*$tv:ident: $bound:path $(, $tv2:ident: $bound2:path)*>, |&$this:ident| { $($rest:tt)* }) => {
        $crate::resource!($target<$($lf:lifetime,)*$tv: $bound$(, $tv2:$bound2)*>, |&$this, _ctx:&()| {
            $($rest)*
        });
    };
    ($target:ident<$($lf:lifetime,)*$tv:ident: $bound:path $(, $tv2:ident: $bound2:path)*>, |&$this:ident, $ctx:ident:&$context:ty| { $($rest:tt)* }) => {
        impl<$($lf,)*$tv$(,$tv2)*> $crate::StaticResource for $target<$($lf,)*$tv$(,$tv2)*>
            where $tv: $bound + Sync + Send,
                  $($tv2: $bound2 + Sync + Send,)*
        {

        }

        impl<$($lf,)*$tv$(,$tv2)*> $crate::StaticKind for $target<$($lf,)*$tv$(,$tv2)*>
            where $tv: $bound + Sync + Send,
                $($tv2: $bound2 + Sync + Send,)*
            {
            fn static_kind() -> $crate::value::Key {
                let raw = $crate::extract_resource_kind!({ $($rest)* }).to_owned();
                $crate::value::Key::from_raw(raw)
            }
        }

        impl<$($lf,)*$tv$(,$tv2)*> $crate::Resource for $target<$($lf,)*$tv$(,$tv2)*>
            where $tv: $bound + Sync + Send,
                $($tv2: $bound2 + Sync + Send,)*
        {
            $crate::resource_body!(|&$this, $ctx:&$context| {
                $($rest)*
            });
        }
    };
}


#[doc(hidden)]
#[macro_export]
macro_rules! resource_body {
    (|&$this:ident, $ctx:ident:&$context:ty| { $($rest:tt)* }) => {
        type Context = $context;
        fn kind<'life0, 'async_trait>(&'life0 $this) -> core::pin::Pin<Box<dyn core::future::Future<Output=$crate::value::Key> + Send + 'async_trait>>
            where 'life0: 'async_trait,
                  Self: 'async_trait,
        {
            let f = async move {
                let raw = $crate::extract_resource_kind!({ $($rest)* }).to_owned();
                $crate::value::Key::from_raw(raw)
            };
            Box::pin(f)
        }

        fn id<'life0, 'async_trait>(&'life0 $this) -> core::pin::Pin<Box<dyn core::future::Future<Output=String> + Send + 'async_trait>>
            where 'life0: 'async_trait,
                  Self: 'async_trait,
        {
            let f = async move {
                $crate::extract_resource_id!({ $($rest)* }).to_string()
            };
            Box::pin(f)
        }

        #[allow(unused_variables)]
        fn to_ident<'life0, 'life1, 'life2, 'async_trait>(
            &'life0 $this,
            executor: &'life1 mut $crate::view::Executor<'life2, Self::Context>,
        ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<$crate::doc::Identifier, $crate::view::ResolveError>> + Send + 'async_trait>>
            where 'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
        {
            let f = async move {
                let mut ident = {
                    let kind = $crate::Resource::kind($this).await;
                    let id = $crate::Resource::id($this).await;

                    $crate::doc::Identifier::new(kind, id)
                };

                {
                    let $ctx = executor.context();
                    let _meta = &mut ident.meta;
                    $crate::expand_resource_impl!(@meta $this, $ctx, _meta, {
                        $($rest)*
                    });
                }

                Ok(ident)
            };
            Box::pin(f)
        }

        #[allow(unused_variables, unused_mut, unused_macros)]
        fn to_object<'life0, 'life1, 'life2, 'async_trait>(
            &'life0 $this,
            executor: &'life1 mut $crate::view::Executor<'life2, Self::Context>,
        ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<$crate::doc::Object, $crate::view::ResolveError>> + Send + 'async_trait>>
            where 'life0: 'async_trait,
                    'life1: 'async_trait,
                    'life2: 'async_trait,
                    Self: 'async_trait,
        {
            /*
            #[allow(dead_code)]
            fn item_kind<T: $crate::Resource>(_: &T) -> $crate::value::Key {
                T::kind()
            }

            #[allow(dead_code)]
            fn iter_kind<'a, I, T>(_: &I) -> $crate::value::Key
            where
                I: Iterator<Item = &'a T>,
                T: $crate::Resource + 'a,
            {
                T::kind()
            }
            */
            let f = async move {
                let kind = $crate::Resource::kind($this).await;
                let mut obj = {
                    let id = $crate::Resource::id($this).await;

                    $crate::doc::Object::new(kind.clone(), id)
                };
                let $ctx = executor.context();

                {
                    let _attrs = &mut obj.attributes;
                    //let _fields = executor.fields(&kind);
                    $crate::expand_resource_impl!(@attrs $this, $ctx, kind, _attrs, executor, {
                        $($rest)*
                    });
                }

                {
                    let _links = &mut obj.links;
                    $crate::expand_resource_impl!(@links $this, $ctx, _links, {
                        $($rest)*
                    });
                }

                {
                    let _meta = &mut obj.meta;
                    $crate::expand_resource_impl!(@meta $this, $ctx, _meta, {
                        $($rest)*
                    });
                }

                {
                    let _related = &mut obj.relationships;
                    //let _fields = executor.fields(&kind);
                    $crate::expand_resource_impl!(@rel $this, $ctx, kind, _related, executor, {
                        $($rest)*
                    });
                }

                Ok(obj)
            };
            Box::pin(f)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_rel_data_impl {
    (@has_one $this:ident, $ctx:ident, $key:ident, $executor:ident, $data:ident, {
        data $data_value:block
        included $include_value:block
    }) => {
        $data = Some($data_value);
        if $executor.key_included(&$key) {
            let mut executor = $executor.fork(&$key);
            macro_rules! convert {
                ($value:expr) => {
                    if let Some(item) = $value {
                        let obj = $crate::Resource::to_object(&item, &mut executor).await?;
                        Some($crate::object_wrap(obj, &executor))
                    } else {
                        None
                    }
                }
            }
            let value = $include_value;
            if let Some(mut item) = value {
                let object = $crate::InternalResolve::resolve_object(&mut item, &mut executor).await?;
                executor.include(object);
            }
        }
        /*
        if $executor.key_included(&$key) {
            let include = $include_value;

            if let Some(item) = include {
                let mut executor = $executor.fork(&$key);
                let object = $crate::Resource::to_object(&item, &mut executor).await?;
                executor.include(object);
            }
        }
        */
    };
    (@has_one $this:ident, $ctx:ident, $key:ident, $executor:ident, $data:ident, {
        data $data_value:block
    }) => {
        let mut _data_ident : Option<$crate::doc::Identifier> = None;
        let mut executor = $executor.fork(&$key);

        macro_rules! convert {
            ($value:expr) => {
                if let Some(item) = $value {
                    _data_ident = Some($crate::Resource::to_ident(&item, &mut executor).await?);
                    if executor.included() {
                        let obj = $crate::Resource::to_object(&item, &mut executor).await?;
                        Some($crate::object_wrap(obj, &executor))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
        let _data_value = $data_value;
        if let Some(data) = _data_ident {
            $data = Some(Some(data));
            if let Some(mut included) = _data_value {
                let object = $crate::InternalResolve::resolve_object(&mut included, &mut executor).await?;
                executor.include(object);
            }
        } else if let Some(mut item) = _data_value {
            $data = Some(Some($crate::InternalResolve::resolve_ident(&item, &mut executor).await?));

            if executor.included() {
                let object = $crate::InternalResolve::resolve_object(&mut item, &mut executor).await?;
                executor.include(object);
            }
    
        } else {
            $data = Some(None)
        }
        /*
        if let Some(item) = $data_value {
            $data = Some(Some($crate::Resource::to_ident(&item, &mut executor).await?));

            if executor.included() {
                let object = $crate::Resource::to_object(&item, &mut executor).await?;
                executor.include(object);
            }
    
        } else {
            $data = Some(None)
        }
        */
    };
    (@has_one $this:ident, $ctx:ident, $key:ident, $executor:ident, $data:ident, {
        included $include_value:block
    }) => {
        if $executor.key_included(&$key) {
            let mut _data_ident = None;
            let mut executor = $executor.fork(&$key);
            macro_rules! convert {
                ($value:expr) => {
                    if let Some(item) = $value {
                        _data_ident = Some($crate::Resource::to_ident(&item, &mut executor).await?);
                        let obj = $crate::Resource::to_object(&item, &mut executor).await?;
                        Some($crate::object_wrap(obj, &executor))
                    } else {
                        None
                    }
                }
            }
            let _include_value = $include_value;
            if let Some(data) = _data_ident {
                $data = Some(Some(data));
                if let Some(mut included) = _include_value {
                    let object = $crate::InternalResolve::resolve_object(&mut included, &mut executor).await?;
                    executor.include(object);
                }
            } else if let Some(mut item) = _include_value {
                $data = Some(Some($crate::InternalResolve::resolve_ident(&item, &mut executor).await?));
                let object = $crate::InternalResolve::resolve_object(&mut item, &mut executor).await?;
                executor.include(object);
            } else {
                $data = Some(None)
            }

            /*
            let _include_value = $include_value;
            if let Some(item) = _include_value {
                let mut executor = $executor.fork(&$key);
                let object = $crate::Resource::to_object(&item, &mut executor).await?;
                executor.include(object);
                $data = Some(Some($crate::Resource::to_ident(&item, &mut executor).await?));
            } else {
                $data = Some(None)
            }
            */
        }
    };


    (@has_many $this:ident, $ctx:ident, $key:ident, $executor:ident, $data:ident, {
        data $data_value:block
        included $include_value:block
    }) => {
        let value = $data_value;
        let data_vec : Vec<$crate::doc::Identifier> = value.collect();
        $data = Some(data_vec);

        if $executor.key_included(&$key) {
            let include = $include_value;
            let mut executor = $executor.fork(&$key);

            for item in include {
                let object = $crate::Resource::to_object(item, &mut executor).await?;
                executor.include(object);
            }
        }
    };
    (@has_many $this:ident, $ctx:ident, $key:ident, $executor:ident, $data:ident, {
        data $data_value:block
    }) => {
        let value = $data_value;
        let mut data_vec = match value.size_hint() {
            (_, Some(size)) => Vec::with_capacity(size),
            _ => Vec::new(),
        };

        let mut executor = $executor.fork(&$key);
        if executor.included() {
            for item in value {
                let object = $crate::Resource::to_object(item, &mut executor).await?;
                let ident = $crate::doc::Identifier::from(&object);
                data_vec.push(ident);
                executor.include(object);
            }
        } else {
            for item in value {
                let ident = $crate::Resource::to_ident(item, &mut executor).await?;
                data_vec.push(ident);
            }
        }
        $data = Some(data_vec);
    };
    (@has_many $this:ident, $ctx:ident, $key:ident, $executor:ident, $data:ident, {
        included $include_value:block
    }) => {
        if $executor.key_included(&$key) {
            let value = $include_value;
            let mut data_vec = match value.size_hint() {
                (_, Some(size)) => Vec::with_capacity(size),
                _ => Vec::new(),
            };
            let mut executor = $executor.fork(&$key);
            for item in value {
                let object = $crate::Resource::to_object(&item, &mut executor).await?;
                let ident = $crate::doc::Identifier::from(&object);
                data_vec.push(ident);
                executor.include(object);
            }
            $data = Some(data_vec);
        }
    };

    // Order data and include blocks
    (@$scope:tt $($args:ident),+, {
        included $include_value:block
        data $data_value:block
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            data $data_value
            included $include_value
        });
    };
    (@$scope:tt $($args:ident),+, {
        data $value:block
        $($rest:tt)+
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            $($rest)+
            data $value
        })
    };
    (@$scope:tt $($args:ident),+, {
        included $value:block
        $($rest:tt)+
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            $($rest)+
            included $value
        })
    };

    (@$scope:tt $($args:ident),+, {}) => {};

    // Ignoring other blocks
    (@$scope:tt $($args:ident),+, {
        meta $key:expr, $value:block
        $($rest:tt)*
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            $($rest)*
        })
    };
    (@$scope:tt $($args:ident),+, {
        link $key:expr, { $($body:tt)* }
        $($rest:tt)*
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            $($rest)*
        })
    };
    (@$scope:tt $($args:ident),+, {
        link $key:expr, $value:block
        $($rest:tt)*
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            $($rest)*
        })
    };

    // Rewrite to blocks
    (@$scope:tt $($args:ident),+, {
        $kwd:ident $value:expr;
        $($rest:tt)*
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            $kwd { $value }
            $($rest)*
        })
    };
    (@$scope:tt $($args:ident),+, {
        $kwd:ident $key:expr, $value:expr;
        $($rest:tt)*
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            $kwd $key, { $value }
            $($rest)*
        })
    };

    /*
    (@$scope:tt $($args:ident),+, {
        $skip:tt
        $($rest:tt)*
    }) => {
        $crate::expand_rel_data_impl!(@$scope $($args),+, {
            $($rest)*
        });
    };

    ($($rest:tt)*) => ();
    */

}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_resource_impl {
    (@attrs $this:ident, $ctx:ident, $kind:ident, $attrs:ident, $executor:ident, {
        attr $key:expr, $value:block
        $($rest:tt)*
    }) => {
        if $executor.field(&$kind, $key) {
            let key = $key.parse::<$crate::value::Key>()?;
            let value = $crate::to_value($value)?;

            $attrs.insert(key, value);
        }

        $crate::expand_resource_impl!(@attrs $this, $ctx, $kind, $attrs, $executor, {
            $($rest)*
        });
    };

    (@attrs $this:ident, $ctx:ident, $($arg:ident),*, { attr $field:ident; $($rest:tt)* }) => {
        $crate::expand_resource_impl!(@attrs $this, $ctx, $($arg),*, {
            attr stringify!($field), &$this.$field;
            $($rest)*
        });
    };

    (@attrs $($arg:ident),*, { attrs $($field:ident),+; $($rest:tt)* }) => {
        $crate::expand_resource_impl!(@attrs $($arg),*, {
            $(attr $field;)+
            $($rest)*
        });
    };

    (@rel $this:ident, $ctx:ident, $kind:ident, $related:ident, $executor:ident, {
        has_many $key:expr, { $($body:tt)* }
        $($rest:tt)*
    }) => {
        if $executor.field(&$kind, $key) {
            let key = $key.parse::<$crate::value::Key>()?;
            $crate::expand_resource_impl!(@has_many $this, ctx, $related, key, $executor, {
                $($body)*
            });
        }

        $crate::expand_resource_impl!(@rel $this, $ctx, $kind, $related, $executor, {
            $($rest)*
        });
    };

    (@rel $this:ident, $ctx:ident, $kind:ident, $related:ident, $executor:ident, {
        has_one $key:expr, { $($body:tt)* }
        $($rest:tt)*
    }) => {
        if $executor.field(&$kind, $key) {
            let key = $key.parse::<$crate::value::Key>()?;
            $crate::expand_resource_impl!(@has_one $this, ctx, $related, key, $executor, {
                $($body)*
            });
        }

        $crate::expand_resource_impl!(@rel $this, $ctx, $kind, $related, $executor, {
            $($rest)*
        });
    };

    (@rel $this:ident, $ctx:ident, $($arg:ident),*, {
        has_many $($field:ident),*;
        $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@rel $this, $ctx, $($arg),*, {
            $(has_many stringify!($field), { data $this.$field.iter(); })*
            $($rest)*
        });
    };

    (@rel $this:ident, $ctx:ident, $($arg:ident),*, {
        has_one $($field:ident),*;
        $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@rel $this, $ctx, $($arg),*, {
            $(has_one stringify!($field), { data $this.$field.as_ref(); })*
            $($rest)*
        });
    };

    (@has_many $this:ident, $ctx:ident, $related:ident, $key:ident, $executor:ident, {
        $($rest:tt)*
    }) => {
        #[allow(unused_assignments)]
        let mut rel = $crate::doc::Relationship::new({
            let mut data : Option<Vec<$crate::doc::Identifier>> = None;
            $crate::expand_rel_data_impl!(@has_many $this, $ctx, $key, $executor, data, {
                $($rest)*
            });
            data.map(|d| d.into())
        });

        {
            let links = &mut rel.links;
            $crate::expand_resource_impl!(@links $this, $ctx, links, {
                $($rest)*
            });
        }

        {
            let _meta = &mut rel.meta;
            $crate::expand_resource_impl!(@meta $this, $ctx, _meta, {
                $($rest)*
            });
        }

        $related.insert($key, rel);
    };

    (@has_one $this:ident, $ctx:ident, $related:ident, $key:ident, $executor:ident, {
        $($rest:tt)*
    }) => {
        #[allow(unused_assignments)]
        let mut rel = $crate::doc::Relationship::new({
            let mut data : Option<Option<$crate::doc::Identifier>> = None;
            $crate::expand_rel_data_impl!(@has_one $this, $ctx, $key, $executor, data, {
                $($rest)*
            });
            data.map(|c| c.into())
        });

        {
            let _links = &mut rel.links;
            $crate::expand_resource_impl!(@links $this, $ctx, _links, {
                $($rest)*
            });
        }

        {
            let _meta = &mut rel.meta;
            $crate::expand_resource_impl!(@meta $this, $ctx, _meta, {
                $($rest)*
            });
        }

        $related.insert($key, rel);
    };

    (@links $this:ident, $ctx:ident, $links:ident, {
        link $key:expr, { $($body:tt)* }
        $($rest:tt)*
    }) => {
        {
            let key = $key.parse::<$crate::value::Key>()?;
            let link = $crate::expand_resource_impl!(@link $this, $ctx, {
                $($body)*
            });

            $links.insert(key, link);
        }

        $crate::expand_resource_impl!(@links $this, $ctx, $links, {
            $($rest)*
        });
    };

    (@links $($args:ident),+, {
        link $key:expr, $value:expr;
        $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@links $($args),+, {
            link $key, { href { $value } }
            $($rest)*
        });
    };

    (@link $this:ident, $ctx:ident, { href $value:block $($rest:tt)* }) => {{
        let mut link = $value.parse::<$crate::doc::Link>()?;

        {
            let _meta = &link.meta;
            $crate::expand_resource_impl!(@meta $this, $ctx, _meta, {
                $($rest)*
            });
        }

        link
    }};

    (@meta $this:ident, $ctx:ident, $meta:ident, {
        meta $key:expr, $value:block
        $($rest:tt)*
    }) => {
        {
            let key = $key.parse::<$crate::value::Key>()?;
            let value = $crate::to_value($value)?;

            $meta.insert(key, value);
        }

        $crate::expand_resource_impl!(@meta $this, $ctx, $meta, {
            $($rest)*
        });
    };

    (@$scope:tt $($args:ident),+, {
        attr $field:ident; $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@$scope $($args),+, {
            $($rest)*
        });
    };

    (@$scope:tt $($args:ident),+, {
        attrs $($field:ident),+; $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@$scope $($args),+, {
            $($rest)*
        });
    };
    (@$scope:tt $($args:ident),+, {
        $kwd:ident $key:expr, { $($body:tt)* }
        $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@$scope $($args),+, {
            $($rest)*
        });
    };
    (@$scope:tt $($args:ident),+, {
        $kwd:ident { $($body:tt)* }
        $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@$scope $($args),+, {
            $($rest)*
        });
    };
    (@$scope:tt $($args:ident),+, {
        $kwd:ident |$conv:ident| { $($body:tt)* }
        $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@$scope $($args),+, {
            $($rest)*
        });
    };

    (@$scope:tt $($args:ident),+, {
        $kwd:ident $value:expr;
        $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@$scope $($args),+, {
            $kwd { $value }
            $($rest)*
        });
    };

    (@$scope:tt $($args:ident),+, {
        $kwd:ident $key:expr, $value:expr;
        $($rest:tt)*
    }) => {
        $crate::expand_resource_impl!(@$scope $($args),+, {
            $kwd $key, { $value }
            $($rest)*
        });
    };
    (@$scope:tt $($args:ident),+, {}) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! extract_resource_id {
    ({ id $value:block $($rest:tt)* }) => { $value };
    ({ id $value:expr; $($rest:tt)* }) => { $value };
    ({ $skip:tt $($rest:tt)* }) => { $crate::extract_resource_id!({ $($rest)* }) };
    ({ $($rest:tt)* }) => ();
}

#[doc(hidden)]
#[macro_export]
macro_rules! extract_resource_kind {
    ({ kind $value:block $($rest:tt)* }) => { $value };
    ({ kind $value:expr; $($rest:tt)* }) => { $value };
    ({ $skip:tt $($rest:tt)* }) => { $crate::extract_resource_kind!({ $($rest)* }) };
    ({ $($rest:tt)* }) => ();
}

#[doc(hidden)]
#[macro_export]
macro_rules! extract_resource_context {
    ({ context $value:ty; $($rest:tt)* }) => { $value };
    ({ $skip:tt $($rest:tt)* }) => { $crate::extract_resource_context!({ $($rest)* }) };
    ({ $($rest:tt)* }) => ();
}

mod test {
    //trace_macros!(true);
    use super::*;

    impl crate::view::Context for String {}

    struct Post {
        id: u64,
        body: String,
        title: String,
        author: Option<User>,
        comments: Vec<Comment<User>>,
    }

    resource!(Post, |&self, ctx: &String| {
        kind "articles";
        id self.id;

        attrs body, title;

        // Define a virtual attribute with an expression
        attr "preview", {
            self.body
                .chars()
                .take(140)
                .collect::<String>()
        }
        attr "resting", {
            ctx.clone()
        }

        // Define a relationship with granular detail
        has_one "author", {            
            // Define relationship links
            link "self", format!("/articles/{}/relationships/author", self.id);
            link "related", format!("/articles/{}/author", self.id);

            // Define arbitrary meta members with a block expression
            meta "read-only", true;

            // Data for has one should be Option<&T> where T: Resource
            data {
                self.author.as_ref()
            }
        }

        has_one "daughter", {            
            // Define relationship links
            link "self", format!("/articles/{}/relationships/author", self.id);
            link "related", format!("/articles/{}/author", self.id);
            included { convert!(self.author.as_ref()) }
            // Define arbitrary meta members with a block expression
            meta "read-only", true;

            // Data for has one should be Option<&T> where T: Resource
            data { Some(Identifier::new(User::static_kind(), "test".into())) }
        }

        has_one "mom", {            
            // Define relationship links
            link "self", format!("/articles/{}/relationships/author", self.id);
            link "related", format!("/articles/{}/author", self.id);
            included { self.author.as_ref() }
            // Define arbitrary meta members with a block expression
            meta "read-only", true;
        }

        has_one "dad", {            
            // Define relationship links
            link "self", format!("/articles/{}/relationships/author", self.id);
            link "related", format!("/articles/{}/author", self.id);
            // Define arbitrary meta members with a block expression
            meta "read-only", true;
        }


        // Define a relationship with granular detail
        has_many "comments", {
            // Data for has one should be an Iterator<Item = &T> where T: Resource
            data self.comments.iter();
            
            // Define relationship links
            link "self", format!("/articles/{}/relationships/comments", self.id);
            link "related", format!("/articles/{}/comments", self.id);
            
            // Define arbitrary meta members with a block expression
            meta "total", {
                self.comments.len()
            }
        }
        has_many "bad_comments", {
            // Define relationship links
            link "self", format!("/articles/{}/relationships/comments", self.id);
            link "related", format!("/articles/{}/comments", self.id);
            
            included { self.comments.iter() }
            // Define arbitrary meta members with a block expression
            meta "total", {
                self.comments.len()
            }

            // Data for has one should be Option<&T> where T: Resource
            data { self.comments.iter().map(|c| Identifier::new(Comment::<User>::static_kind(), "muh!".to_string() ) ) }
        }
        has_many "good_comments", {
            // Define relationship links
            link "self", format!("/articles/{}/relationships/comments", self.id);
            link "related", format!("/articles/{}/comments", self.id);
            
            included { self.comments.iter() }
            // Define arbitrary meta members with a block expression
            meta "total", {
                self.comments.len()
            }
        }
        has_many "meh_comments", {
            // Define relationship links
            link "self", format!("/articles/{}/relationships/comments", self.id);
            link "related", format!("/articles/{}/comments", self.id);
            
            // Define arbitrary meta members with a block expression
            meta "total", {
                self.comments.len()
            }
        }

        
        // You can also define links with granular details as well
        link "self", {
            href format!("/articles/{}", self.id);
        }

        // Define arbitrary meta members an expression
        meta "copyright", self.author.as_ref().map(|user| {
            format!("© 2017 {}", user.full_name())
        });
    });
    
    #[derive(Clone)]
    struct User;

    impl User {
        fn full_name(&self) -> String {
            String::new()
        }
    }

    resource!(User, |&self| {
        kind "users";
        id String::new();
    });

    struct Comment<T>(T);

    resource!(Comment<T:StaticResource>, |&self, _ctx:&()| {
        kind format!("comment-{}", T::static_kind());
        id self.0.id().await;
    });
}