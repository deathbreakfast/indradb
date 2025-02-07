#![no_main]

use std::collections::HashMap;

use arbitrary::{Arbitrary, Unstructured};
use indradb::{Datastore, MemoryDatastore, RocksdbDatastore, Transaction};
use libfuzzer_sys::fuzz_target;
use tempfile::tempdir;

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub enum Op {
    BulkInsert(Vec<BulkInsertItem>),
    CreateVertex(Vertex),
    GetVertices(VertexQuery),
    DeleteVertices(VertexQuery),
    GetVertexCount,
    CreateEdge(EdgeKey),
    GetEdges(EdgeQuery),
    DeleteEdges(EdgeQuery),
    GetEdgeCount(Uuid, Option<Identifier>, EdgeDirection),
    GetVertexProperties(VertexPropertyQuery),
    GetAllVertexProperties(VertexQuery),
    SetVertexProperties(VertexPropertyQuery, Json),
    DeleteVertexProperties(VertexPropertyQuery),
    GetEdgeProperties(EdgePropertyQuery),
    GetAllEdgeProperties(EdgeQuery),
    SetEdgeProperties(EdgePropertyQuery, Json),
    DeleteEdgeProperties(EdgePropertyQuery),
    IndexProperty(Identifier),
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub enum BulkInsertItem {
    Vertex(Vertex),
    Edge(EdgeKey),
    VertexProperty(Uuid, Identifier, Json),
    EdgeProperty(EdgeKey, Identifier, Json),
}

impl Into<indradb::BulkInsertItem> for BulkInsertItem {
    fn into(self) -> indradb::BulkInsertItem {
        match self {
            BulkInsertItem::Vertex(vertex) => indradb::BulkInsertItem::Vertex(vertex.into()),
            BulkInsertItem::Edge(key) => indradb::BulkInsertItem::Edge(key.into()),
            BulkInsertItem::VertexProperty(id, name, value) => {
                indradb::BulkInsertItem::VertexProperty(id.into(), name.into(), value.into())
            }
            BulkInsertItem::EdgeProperty(key, name, value) => {
                indradb::BulkInsertItem::EdgeProperty(key.into(), name.into(), value.into())
            }
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub id: Uuid,
    pub t: Identifier,
}

impl Into<indradb::Vertex> for Vertex {
    fn into(self) -> indradb::Vertex {
        indradb::Vertex::with_id(self.id.into(), self.t.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier(indradb::Identifier);

impl<'a> Arbitrary<'a> for Identifier {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let s: String = u.arbitrary()?;

        if s.is_empty() {
            return Err(arbitrary::Error::NotEnoughData);
        }

        let t = indradb::Identifier::new(s).map_err(|_| arbitrary::Error::IncorrectFormat)?;
        Ok(Self { 0: t })
    }
}

impl Into<indradb::Identifier> for Identifier {
    fn into(self) -> indradb::Identifier {
        self.0
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub enum EdgeDirection {
    Outbound,
    Inbound,
}

impl Into<indradb::EdgeDirection> for EdgeDirection {
    fn into(self) -> indradb::EdgeDirection {
        match self {
            EdgeDirection::Outbound => indradb::EdgeDirection::Outbound,
            EdgeDirection::Inbound => indradb::EdgeDirection::Inbound,
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub enum VertexQuery {
    Range(RangeVertexQuery),
    Specific(SpecificVertexQuery),
    Pipe(PipeVertexQuery),
    PropertyPresence(PropertyPresenceVertexQuery),
    PropertyValue(PropertyValueVertexQuery),
    PipePropertyPresence(PipePropertyPresenceVertexQuery),
    PipePropertyValue(PipePropertyValueVertexQuery),
}

impl Into<indradb::VertexQuery> for VertexQuery {
    fn into(self) -> indradb::VertexQuery {
        match self {
            VertexQuery::Range(q) => indradb::VertexQuery::Range(q.into()),
            VertexQuery::Specific(q) => indradb::VertexQuery::Specific(q.into()),
            VertexQuery::Pipe(q) => indradb::VertexQuery::Pipe(q.into()),
            VertexQuery::PropertyPresence(q) => indradb::VertexQuery::PropertyPresence(q.into()),
            VertexQuery::PropertyValue(q) => indradb::VertexQuery::PropertyValue(q.into()),
            VertexQuery::PipePropertyPresence(q) => indradb::VertexQuery::PipePropertyPresence(q.into()),
            VertexQuery::PipePropertyValue(q) => indradb::VertexQuery::PipePropertyValue(q.into()),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct RangeVertexQuery {
    pub limit: u32,
    pub t: Option<Identifier>,
    pub start_id: Option<Uuid>,
}

impl Into<indradb::RangeVertexQuery> for RangeVertexQuery {
    fn into(self) -> indradb::RangeVertexQuery {
        indradb::RangeVertexQuery {
            limit: self.limit,
            t: self.t.map(|t| t.into()),
            start_id: self.start_id.map(|id| id.into()),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct SpecificVertexQuery {
    pub ids: Vec<Uuid>,
}

impl Into<indradb::SpecificVertexQuery> for SpecificVertexQuery {
    fn into(self) -> indradb::SpecificVertexQuery {
        indradb::SpecificVertexQuery {
            ids: self.ids.into_iter().map(|i| i.into()).collect(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct PipeVertexQuery {
    pub inner: Box<EdgeQuery>,
    pub direction: EdgeDirection,
    pub limit: u32,
    pub t: Option<Identifier>,
}

impl Into<indradb::PipeVertexQuery> for PipeVertexQuery {
    fn into(self) -> indradb::PipeVertexQuery {
        indradb::PipeVertexQuery {
            inner: Box::new((*self.inner).into()),
            direction: self.direction.into(),
            limit: self.limit,
            t: self.t.map(|t| t.into()),
        }
    }
}

#[derive(Arbitrary, PartialEq, Clone, Debug)]
pub struct PropertyPresenceVertexQuery {
    pub name: Identifier,
}

impl Into<indradb::PropertyPresenceVertexQuery> for PropertyPresenceVertexQuery {
    fn into(self) -> indradb::PropertyPresenceVertexQuery {
        indradb::PropertyPresenceVertexQuery {
            name: self.name.into(),
        }
    }
}

#[derive(Arbitrary, PartialEq, Clone, Debug)]
pub struct PropertyValueVertexQuery {
    pub name: Identifier,
    pub value: Json,
}

impl Into<indradb::PropertyValueVertexQuery> for PropertyValueVertexQuery {
    fn into(self) -> indradb::PropertyValueVertexQuery {
        indradb::PropertyValueVertexQuery {
            name: self.name.into(),
            value: self.value.into(),
        }
    }
}

#[derive(Arbitrary, PartialEq, Clone, Debug)]
pub struct PipePropertyPresenceVertexQuery {
    pub inner: Box<VertexQuery>,
    pub name: Identifier,
    pub exists: bool,
}

impl Into<indradb::PipePropertyPresenceVertexQuery> for PipePropertyPresenceVertexQuery {
    fn into(self) -> indradb::PipePropertyPresenceVertexQuery {
        indradb::PipePropertyPresenceVertexQuery {
            inner: Box::new((*self.inner).into()),
            name: self.name.into(),
            exists: self.exists,
        }
    }
}

#[derive(Arbitrary, PartialEq, Clone, Debug)]
pub struct PipePropertyValueVertexQuery {
    pub inner: Box<VertexQuery>,
    pub name: Identifier,
    pub value: Json,
    pub equal: bool,
}

impl Into<indradb::PipePropertyValueVertexQuery> for PipePropertyValueVertexQuery {
    fn into(self) -> indradb::PipePropertyValueVertexQuery {
        indradb::PipePropertyValueVertexQuery {
            inner: Box::new((*self.inner).into()),
            name: self.name.into(),
            value: self.value.into(),
            equal: self.equal,
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct VertexPropertyQuery {
    pub inner: VertexQuery,
    pub name: Identifier,
}

impl Into<indradb::VertexPropertyQuery> for VertexPropertyQuery {
    fn into(self) -> indradb::VertexPropertyQuery {
        indradb::VertexPropertyQuery {
            inner: self.inner.into(),
            name: self.name.into(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub enum EdgeQuery {
    Specific(SpecificEdgeQuery),
    Pipe(PipeEdgeQuery),
    PropertyPresence(PropertyPresenceEdgeQuery),
    PropertyValue(PropertyValueEdgeQuery),
    PipePropertyPresence(PipePropertyPresenceEdgeQuery),
    PipePropertyValue(PipePropertyValueEdgeQuery),
}

impl Into<indradb::EdgeQuery> for EdgeQuery {
    fn into(self) -> indradb::EdgeQuery {
        match self {
            EdgeQuery::Specific(specific) => indradb::EdgeQuery::Specific(specific.into()),
            EdgeQuery::Pipe(pipe) => indradb::EdgeQuery::Pipe(pipe.into()),
            EdgeQuery::PropertyPresence(q) => indradb::EdgeQuery::PropertyPresence(q.into()),
            EdgeQuery::PropertyValue(q) => indradb::EdgeQuery::PropertyValue(q.into()),
            EdgeQuery::PipePropertyPresence(q) => indradb::EdgeQuery::PipePropertyPresence(q.into()),
            EdgeQuery::PipePropertyValue(q) => indradb::EdgeQuery::PipePropertyValue(q.into()),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct SpecificEdgeQuery {
    pub keys: Vec<EdgeKey>,
}

impl Into<indradb::SpecificEdgeQuery> for SpecificEdgeQuery {
    fn into(self) -> indradb::SpecificEdgeQuery {
        indradb::SpecificEdgeQuery {
            keys: self.keys.into_iter().map(|i| i.into()).collect(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct PipeEdgeQuery {
    pub inner: Box<VertexQuery>,
    pub direction: EdgeDirection,
    pub limit: u32,
    pub t: Option<Identifier>,
    pub high: Option<DateTime>,
    pub low: Option<DateTime>,
}

impl Into<indradb::PipeEdgeQuery> for PipeEdgeQuery {
    fn into(self) -> indradb::PipeEdgeQuery {
        indradb::PipeEdgeQuery {
            inner: Box::new((*self.inner).into()),
            direction: self.direction.into(),
            limit: self.limit,
            t: self.t.map(|t| t.into()),
            high: self.high.map(|d| d.into()),
            low: self.low.map(|d| d.into()),
        }
    }
}

#[derive(Arbitrary, PartialEq, Clone, Debug)]
pub struct PropertyPresenceEdgeQuery {
    pub name: Identifier,
}

impl Into<indradb::PropertyPresenceEdgeQuery> for PropertyPresenceEdgeQuery {
    fn into(self) -> indradb::PropertyPresenceEdgeQuery {
        indradb::PropertyPresenceEdgeQuery {
            name: self.name.into(),
        }
    }
}

#[derive(Arbitrary, PartialEq, Clone, Debug)]
pub struct PropertyValueEdgeQuery {
    pub name: Identifier,
    pub value: Json,
}

impl Into<indradb::PropertyValueEdgeQuery> for PropertyValueEdgeQuery {
    fn into(self) -> indradb::PropertyValueEdgeQuery {
        indradb::PropertyValueEdgeQuery {
            name: self.name.into(),
            value: self.value.into(),
        }
    }
}

#[derive(Arbitrary, PartialEq, Clone, Debug)]
pub struct PipePropertyPresenceEdgeQuery {
    pub inner: Box<EdgeQuery>,
    pub name: Identifier,
    pub exists: bool,
}

impl Into<indradb::PipePropertyPresenceEdgeQuery> for PipePropertyPresenceEdgeQuery {
    fn into(self) -> indradb::PipePropertyPresenceEdgeQuery {
        indradb::PipePropertyPresenceEdgeQuery {
            inner: Box::new((*self.inner).into()),
            name: self.name.into(),
            exists: self.exists,
        }
    }
}

#[derive(Arbitrary, PartialEq, Clone, Debug)]
pub struct PipePropertyValueEdgeQuery {
    pub inner: Box<EdgeQuery>,
    pub name: Identifier,
    pub value: Json,
    pub equal: bool,
}

impl Into<indradb::PipePropertyValueEdgeQuery> for PipePropertyValueEdgeQuery {
    fn into(self) -> indradb::PipePropertyValueEdgeQuery {
        indradb::PipePropertyValueEdgeQuery {
            inner: Box::new((*self.inner).into()),
            name: self.name.into(),
            value: self.value.into(),
            equal: self.equal,
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct EdgePropertyQuery {
    pub inner: EdgeQuery,
    pub name: Identifier,
}

impl Into<indradb::EdgePropertyQuery> for EdgePropertyQuery {
    fn into(self) -> indradb::EdgePropertyQuery {
        indradb::EdgePropertyQuery {
            inner: self.inner.into(),
            name: self.name.into(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct VertexProperty {
    pub id: Uuid,
    pub value: Json,
}

impl Into<indradb::VertexProperty> for VertexProperty {
    fn into(self) -> indradb::VertexProperty {
        indradb::VertexProperty {
            id: self.id.into(),
            value: self.value.into(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct NamedProperty {
    pub name: Identifier,
    pub value: Json,
}

impl Into<indradb::NamedProperty> for NamedProperty {
    fn into(self) -> indradb::NamedProperty {
        indradb::NamedProperty {
            name: self.name.into(),
            value: self.value.into(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct VertexProperties {
    pub vertex: Vertex,
    pub props: Vec<NamedProperty>,
}

impl Into<indradb::VertexProperties> for VertexProperties {
    fn into(self) -> indradb::VertexProperties {
        indradb::VertexProperties {
            vertex: self.vertex.into(),
            props: self.props.into_iter().map(|p| p.into()).collect(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct EdgeProperties {
    pub edge: Edge,
    pub props: Vec<NamedProperty>,
}

impl Into<indradb::EdgeProperties> for EdgeProperties {
    fn into(self) -> indradb::EdgeProperties {
        indradb::EdgeProperties {
            edge: self.edge.into(),
            props: self.props.into_iter().map(|p| p.into()).collect(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct EdgeProperty {
    pub key: EdgeKey,
    pub value: Json,
}

impl Into<indradb::EdgeProperty> for EdgeProperty {
    fn into(self) -> indradb::EdgeProperty {
        indradb::EdgeProperty {
            key: self.key.into(),
            value: self.value.into(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct EdgeKey {
    pub outbound_id: Uuid,
    pub t: Identifier,
    pub inbound_id: Uuid,
}

impl Into<indradb::EdgeKey> for EdgeKey {
    fn into(self) -> indradb::EdgeKey {
        indradb::EdgeKey {
            outbound_id: self.outbound_id.into(),
            t: self.t.into(),
            inbound_id: self.inbound_id.into(),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub struct Edge {
    pub key: EdgeKey,
    pub created_datetime: DateTime,
}

impl Into<indradb::Edge> for Edge {
    fn into(self) -> indradb::Edge {
        indradb::Edge {
            key: self.key.into(),
            created_datetime: self.created_datetime.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Uuid(uuid::Uuid);

impl<'a> Arbitrary<'a> for Uuid {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            0: uuid::Uuid::from_u128(u.arbitrary()?),
        })
    }
}

impl Into<uuid::Uuid> for Uuid {
    fn into(self) -> uuid::Uuid {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl Into<chrono::DateTime<chrono::Utc>> for DateTime {
    fn into(self) -> chrono::DateTime<chrono::Utc> {
        self.0
    }
}

impl<'a> Arbitrary<'a> for DateTime {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let t: i64 = u.arbitrary()?;
        let n: u32 = u.arbitrary()?;
        let naive = chrono::NaiveDateTime::from_timestamp_opt(t, n).ok_or(arbitrary::Error::IncorrectFormat)?;
        let dt = chrono::DateTime::<chrono::Utc>::from_utc(naive, chrono::Utc);
        Ok(Self { 0: dt })
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

impl Into<serde_json::Value> for Json {
    fn into(self) -> serde_json::Value {
        match self {
            Json::Null => serde_json::Value::Null,
            Json::Bool(b) => serde_json::Value::Bool(b),
            Json::Number(n) => serde_json::Value::Number(n.into()),
            Json::String(s) => serde_json::Value::String(s),
            Json::Array(v) => serde_json::Value::Array(v.into_iter().map(|i| i.into()).collect()),
            Json::Object(o) => {
                let mut m = serde_json::Map::new();

                for (k, v) in o.into_iter() {
                    m.insert(k, v.into());
                }

                serde_json::Value::Object(m)
            }
        }
    }
}

#[derive(Arbitrary, Clone, Debug, PartialEq)]
pub enum JsonNumber {
    PosInt(u64),
    NegInt(i64),
    Float(FiniteFloat),
}

impl Into<serde_json::Number> for JsonNumber {
    fn into(self) -> serde_json::Number {
        match self {
            JsonNumber::PosInt(n) => n.into(),
            JsonNumber::NegInt(n) => n.into(),
            JsonNumber::Float(n) => serde_json::Number::from_f64(n.into()).unwrap(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FiniteFloat(f64);

impl<'a> Arbitrary<'a> for FiniteFloat {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let f: f64 = u.arbitrary()?;

        if f.is_finite() {
            Ok(Self { 0: f })
        } else {
            Err(arbitrary::Error::IncorrectFormat)
        }
    }
}

impl Into<f64> for FiniteFloat {
    fn into(self) -> f64 {
        self.0
    }
}

macro_rules! cmp {
    ($v1:expr, $v2:expr) => {
        match ($v1, $v2) {
            (Ok(v1), Ok(v2)) => {
                assert_eq!(v1, v2);
            }
            (v1, v2) => {
                assert_eq!(format!("{:?}", v1), format!("{:?}", v2));
            }
        }
    };
}

fuzz_target!(|ops: Vec<Op>| {
    let d1 = MemoryDatastore::default();
    let rocksdb_dir = tempdir().unwrap();
    let d2 = RocksdbDatastore::new(rocksdb_dir.path(), Some(1)).unwrap();

    let t1 = d1.transaction().unwrap();
    let t2 = d2.transaction().unwrap();

    for op in ops {
        match op {
            Op::BulkInsert(items) => {
                let items: Vec<indradb::BulkInsertItem> = items.into_iter().map(|i| i.into()).collect();
                let v1 = d1.bulk_insert(items.clone().into_iter());
                let v2 = d2.bulk_insert(items.into_iter());
                cmp!(v1, v2);
            }
            Op::CreateVertex(vertex) => {
                let vertex = vertex.into();
                let v1 = t1.create_vertex(&vertex);
                let v2 = t2.create_vertex(&vertex);
                cmp!(v1, v2);
            }
            Op::GetVertices(q) => {
                let q: indradb::VertexQuery = q.into();
                let v1 = t1.get_vertices(q.clone());
                let v2 = t2.get_vertices(q);
                cmp!(v1, v2);
            }
            Op::DeleteVertices(q) => {
                let q: indradb::VertexQuery = q.into();
                let v1 = t1.delete_vertices(q.clone());
                let v2 = t2.delete_vertices(q);
                cmp!(v1, v2);
            }
            Op::GetVertexCount => {
                let v1 = t1.get_vertex_count();
                let v2 = t2.get_vertex_count();
                cmp!(v1, v2);
            }
            Op::CreateEdge(key) => {
                let key: indradb::EdgeKey = key.into();
                let v1 = t1.create_edge(&key);
                let v2 = t2.create_edge(&key);
                cmp!(v1, v2);
            }
            Op::GetEdges(q) => {
                let q: indradb::EdgeQuery = q.into();
                let v1 = t1.get_edges(q.clone());
                let v2 = t2.get_edges(q);
                cmp!(v1, v2);
            }
            Op::DeleteEdges(q) => {
                let q: indradb::EdgeQuery = q.into();
                let v1 = t1.delete_edges(q.clone());
                let v2 = t2.delete_edges(q);
                cmp!(v1, v2);
            }
            Op::GetEdgeCount(id, t, direction) => {
                let id: uuid::Uuid = id.into();
                let t: Option<indradb::Identifier> = t.map(|t| t.into());
                let direction: indradb::EdgeDirection = direction.into();
                let v1 = t1.get_edge_count(id, t.as_ref(), direction);
                let v2 = t2.get_edge_count(id, t.as_ref(), direction);
                cmp!(v1, v2);
            }
            Op::GetVertexProperties(q) => {
                let q: indradb::VertexPropertyQuery = q.into();
                let v1 = t1.get_vertex_properties(q.clone());
                let v2 = t2.get_vertex_properties(q);
                cmp!(v1, v2);
            }
            Op::GetAllVertexProperties(q) => {
                let q: indradb::VertexQuery = q.into();
                let v1 = t1.get_all_vertex_properties(q.clone());
                let v2 = t2.get_all_vertex_properties(q);
                cmp!(v1, v2);
            }
            Op::SetVertexProperties(q, value) => {
                let q: indradb::VertexPropertyQuery = q.into();
                let value: serde_json::Value = value.into();
                let v1 = t1.set_vertex_properties(q.clone(), value.clone());
                let v2 = t2.set_vertex_properties(q, value);
                cmp!(v1, v2);
            }
            Op::DeleteVertexProperties(q) => {
                let q: indradb::VertexPropertyQuery = q.into();
                let v1 = t1.delete_vertex_properties(q.clone());
                let v2 = t2.delete_vertex_properties(q);
                cmp!(v1, v2);
            }
            Op::GetEdgeProperties(q) => {
                let q: indradb::EdgePropertyQuery = q.into();
                let v1 = t1.get_edge_properties(q.clone());
                let v2 = t2.get_edge_properties(q);
                cmp!(v1, v2);
            }
            Op::GetAllEdgeProperties(q) => {
                let q: indradb::EdgeQuery = q.into();
                let v1 = t1.get_all_edge_properties(q.clone());
                let v2 = t2.get_all_edge_properties(q);
                cmp!(v1, v2);
            }
            Op::SetEdgeProperties(q, value) => {
                let q: indradb::EdgePropertyQuery = q.into();
                let value: serde_json::Value = value.into();
                let v1 = t1.set_edge_properties(q.clone(), value.clone());
                let v2 = t2.set_edge_properties(q, value);
                cmp!(v1, v2);
            }
            Op::DeleteEdgeProperties(q) => {
                let q: indradb::EdgePropertyQuery = q.into();
                let v1 = t1.delete_edge_properties(q.clone());
                let v2 = t2.delete_edge_properties(q);
                cmp!(v1, v2);
            },
            Op::IndexProperty(t) => {
                let v1 = d1.index_property(t.clone());
                let v2 = d2.index_property(t);
                cmp!(v1, v2);
            }
        }
    }
});
