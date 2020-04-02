// Copying implementation from diesel:master as .into_boxed() still not released with 1.4.4
// As soon as 1.5 released this code is redundant
use diesel::backend::Backend;
use diesel::connection::Connection;
use diesel::deserialize::QueryableByName;
use diesel::query_builder::SqlQuery;
use diesel::query_builder::{AstPass, QueryFragment, QueryId};
use diesel::query_dsl::{LoadQuery, RunQueryDsl};
use diesel::result::QueryResult;
use diesel::serialize::ToSql;
use diesel::sql_types::HasSqlType;

pub(crate) trait IntoBoxed<'f, DB: Backend, Query> {
    fn into_boxed(self) -> BoxedSqlQuery<'f, DB, Query>;
}

impl<'f, DB: Backend> IntoBoxed<'f, DB, SqlQuery> for SqlQuery {
    fn into_boxed(self) -> BoxedSqlQuery<'f, DB, Self> {
        BoxedSqlQuery::new(self)
    }
}

#[must_use = "Queries are only executed when calling `load`, `get_result`, or similar."]
/// See [`SqlQuery::into_boxed`].
///
/// [`SqlQuery::into_boxed`]: ./struct.SqlQuery.html#method.into_boxed
#[allow(missing_debug_implementations)]
pub struct BoxedSqlQuery<'f, DB: Backend, Query> {
    query: Query,
    sql: String,
    binds: Vec<Box<dyn Fn(AstPass<DB>) -> QueryResult<()> + 'f>>,
}

impl<'f, DB: Backend, Query> BoxedSqlQuery<'f, DB, Query> {
    pub(crate) fn new(query: Query) -> Self {
        BoxedSqlQuery {
            query,
            sql: "".to_string(),
            binds: vec![],
        }
    }

    /// See [`SqlQuery::bind`].
    ///
    /// [`SqlQuery::bind`]: ./struct.SqlQuery.html#method.bind
    pub fn bind<BindSt, Value>(mut self, b: Value) -> Self
    where
        DB: HasSqlType<BindSt>,
        Value: ToSql<BindSt, DB> + 'f,
    {
        self.binds
            .push(Box::new(move |mut out| out.push_bind_param_value_only(&b)));
        self
    }

    /// See [`SqlQuery::sql`].
    ///
    /// [`SqlQuery::sql`]: ./struct.SqlQuery.html#method.sql
    pub fn sql<T: AsRef<str>>(mut self, sql: T) -> Self {
        self.sql += sql.as_ref();
        self
    }
}

impl<DB, Query> QueryFragment<DB> for BoxedSqlQuery<'_, DB, Query>
where
    DB: Backend,
    Query: QueryFragment<DB>,
{
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(&self.sql);

        for b in &self.binds {
            b(out.reborrow())?;
        }
        Ok(())
    }
}

impl<DB: Backend, Query> QueryId for BoxedSqlQuery<'_, DB, Query> {
    type QueryId = ();

    const HAS_STATIC_QUERY_ID: bool = false;
}

impl<Conn, T, Query> LoadQuery<Conn, T> for BoxedSqlQuery<'_, Conn::Backend, Query>
where
    Conn: Connection,
    T: QueryableByName<Conn::Backend>,
    Self: QueryFragment<Conn::Backend> + QueryId,
{
    fn internal_load(self, conn: &Conn) -> QueryResult<Vec<T>> {
        conn.query_by_name(&self)
    }
}

impl<Conn: Connection, Query> RunQueryDsl<Conn> for BoxedSqlQuery<'_, Conn::Backend, Query> {}
