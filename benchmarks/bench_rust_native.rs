//! Native Rust benchmark for parse + validate (no PyO3).
//!
//! This isolates the cost of the Rust GraphQL pipeline from any Python FFI.
//! If `bench_rustberry_vs_core.py` shows Rust parse is "only" ~2-3x faster
//! than graphql-core, this file tells us whether the gap is in apollo-rs
//! itself or in the PyO3 boundary.
//!
//! ## Wiring
//!
//! The instructions for this benchmark suite forbid touching `Cargo.toml`,
//! so this file is *not* registered as a `[[bench]]` target by default. To
//! run it, add the following to the root `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! criterion = "0.5"
//!
//! [[bench]]
//! name = "bench_rust_native"
//! path = "benchmarks/bench_rust_native.rs"
//! harness = false
//! ```
//!
//! Then: `cargo bench --bench bench_rust_native`.
//!
//! Note: rustberry's `[lib]` is a `cdylib`, so `cargo bench` may complain
//! about the missing `rlib` crate type. Either add `crate-type = ["cdylib",
//! "rlib"]` *temporarily* (don't commit) or run the bench from a fresh
//! cargo workspace that depends on apollo-compiler/apollo-parser directly.

use std::hint::black_box;

use apollo_compiler::{ExecutableDocument, Schema};
use apollo_compiler::validation::Valid;
use criterion::{criterion_group, criterion_main, Criterion};

const SCHEMA_SDL: &str = r#"
schema { query: Query mutation: Mutation }

type Query {
  hello: String
  user(id: ID!): User
  users(first: Int = 10, filter: UserFilter): [User!]!
  search(query: String!, kind: SearchKind = USER): [SearchResult!]!
  node(id: ID!): Node
}

type Mutation {
  createUser(input: CreateUserInput!): User!
  updateUser(id: ID!, input: UpdateUserInput!): User!
}

interface Node { id: ID! }
union SearchResult = User | Post | Comment
enum SearchKind { USER POST COMMENT }

input UserFilter { name: String  active: Boolean  tags: [String!] }
input CreateUserInput { name: String!  email: String!  age: Int  tags: [String!]  preferences: PreferencesInput }
input UpdateUserInput { name: String  email: String  age: Int  preferences: PreferencesInput }
input PreferencesInput { theme: String  notifications: Boolean }

type User implements Node {
  id: ID!  name: String!  email: String!  age: Int  active: Boolean
  tags: [String!]!  posts(first: Int = 5): [Post!]!  comments: [Comment!]!
  followers: [User!]!  following: [User!]!  preferences: Preferences
  legacyName: String @deprecated(reason: "use name")
}
type Preferences { theme: String  notifications: Boolean  language: String }
type Post implements Node {
  id: ID!  title: String!  body: String!  author: User!
  comments: [Comment!]!  tags: [String!]!  createdAt: String!
}
type Comment implements Node {
  id: ID!  body: String!  author: User!  post: Post!  replies: [Comment!]!
}
"#;

const Q_SIMPLE: &str = "{ hello }";

const Q_MEDIUM: &str = r#"
query GetUser($id: ID!, $includeEmail: Boolean = true) {
  user(id: $id) {
    id name email @include(if: $includeEmail)
    posts(first: 3) { ...PostSummary }
  }
}
fragment PostSummary on Post { id title createdAt }
"#;

const Q_COMPLEX: &str = r#"
query Dashboard($id: ID!, $count: Int = 10, $term: String!) {
  primary: user(id: $id) {
    id displayName: name email age active tags
    preferences { theme notifications language }
    posts(first: $count) {
      id title body createdAt tags
      author { id name followers { id name } }
      comments { id body author { id name } replies { id body } }
    }
    followers { id name email active tags }
    following { id name email active tags }
  }
  results: search(query: $term, kind: USER) {
    __typename
    ... on User { id name tags }
    ... on Post { id title tags }
    ... on Comment { id body }
  }
}
"#;

/// Pre-build the schema once per benchmark group; schema parsing dominates
/// otherwise and obscures the parse/validate cost we care about.
fn schema() -> Valid<Schema> {
    let s = Schema::parse(SCHEMA_SDL, "schema.graphql").expect("schema parse");
    Valid::assume_valid(s)
}

fn bench_parse(c: &mut Criterion) {
    let schema = schema();
    let mut group = c.benchmark_group("parse");
    for (name, src) in [("Simple", Q_SIMPLE), ("Medium", Q_MEDIUM), ("Complex", Q_COMPLEX)] {
        group.bench_function(name, |b| {
            b.iter(|| {
                let doc = ExecutableDocument::parse(
                    black_box(&schema),
                    black_box(src),
                    "q.graphql",
                )
                .expect("parse");
                black_box(doc)
            })
        });
    }
    group.finish();
}

fn bench_parse_validate(c: &mut Criterion) {
    let schema = schema();
    let mut group = c.benchmark_group("parse_validate");
    for (name, src) in [("Simple", Q_SIMPLE), ("Medium", Q_MEDIUM), ("Complex", Q_COMPLEX)] {
        group.bench_function(name, |b| {
            b.iter(|| {
                let doc = ExecutableDocument::parse(
                    black_box(&schema),
                    black_box(src),
                    "q.graphql",
                )
                .expect("parse");
                let valid = doc.validate(black_box(&schema)).expect("validate");
                black_box(valid)
            })
        });
    }
    group.finish();
}

fn bench_validate_only(c: &mut Criterion) {
    let schema = schema();
    let mut group = c.benchmark_group("validate_only");
    for (name, src) in [("Simple", Q_SIMPLE), ("Medium", Q_MEDIUM), ("Complex", Q_COMPLEX)] {
        // Pre-parse once; only validation is timed.
        let doc = ExecutableDocument::parse(&schema, src, "q.graphql").expect("parse");
        group.bench_function(name, |b| {
            b.iter(|| {
                let valid = doc.clone().validate(black_box(&schema)).expect("validate");
                black_box(valid)
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse, bench_parse_validate, bench_validate_only);
criterion_main!(benches);
