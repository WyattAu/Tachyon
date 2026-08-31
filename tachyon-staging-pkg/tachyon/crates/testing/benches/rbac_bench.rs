//! Performance benchmarks for RBAC operations
//!
//! Benchmarks permission checking, enforcer operations, and authorization operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tachyon_core::types::user::UserRole;
use tachyon_core::{SessionId, UserId};
use tachyon_rbac::types::{AccessRequest, Action, AuthContext, Effect};
use tachyon_rbac::{Enforcer, Permission, PermissionChecker, resource, subject};

fn bench_permission_operations(c: &mut Criterion) {
    c.bench_function("permission_creation", |b| {
        b.iter(|| {
            let _ = Permission::new("read_docs", "document:*", "read", Effect::Allow);
        });
    });

    c.bench_function("permission_matches", |b| {
        let permission = Permission::new("read_docs", "document:*", "read", Effect::Allow);
        let res = resource("document", "doc123");
        let action = Action::new("read");

        b.iter(|| {
            let _ = black_box(permission.matches(&res, &action));
        });
    });

    c.bench_function("permission_checker", |b| {
        let mut checker = PermissionChecker::new();
        let permission = Permission::new("read_docs", "document:*", "read", Effect::Allow);
        checker.add_permission(permission);

        let res = resource("document", "doc123");
        let action = Action::new("read");
        let sub = subject("user", "user1");

        b.iter(|| {
            let _ = black_box(checker.check_permission(&sub, &res, &action));
        });
    });
}

fn bench_enforcer_operations(c: &mut Criterion) {
    c.bench_function("enforcer_creation", |b| {
        b.iter(|| {
            let _ = Enforcer::new();
        });
    });

    c.bench_function("enforcer_authorize", |b| {
        let mut enforcer = Enforcer::new();
        let user_id = UserId::new();
        let session_id = SessionId::new();
        let sub = subject("user", "user1");
        let res = resource("document", "doc123");
        let action = Action::new("read");
        let context = AuthContext::new(user_id, session_id);
        let request = AccessRequest::new(sub, res, action, context);

        b.iter(|| {
            let _ = black_box(enforcer.authorize(&request));
        });
    });
}

fn bench_user_role_operations(c: &mut Criterion) {
    c.bench_function("user_role_creation", |b| {
        b.iter(|| {
            let _ = UserRole::Reader;
        });
    });

    c.bench_function("user_role_permission_level", |b| {
        let role = UserRole::Admin;

        b.iter(|| {
            let _ = black_box(role.permission_level());
        });
    });

    c.bench_function("user_role_can_perform", |b| {
        let role = UserRole::Editor;
        let action = tachyon_core::types::user::UserAction::Write;

        b.iter(|| {
            let _ = black_box(role.can_perform(action));
        });
    });
}

criterion_group!(
    benches,
    bench_permission_operations,
    bench_enforcer_operations,
    bench_user_role_operations
);

criterion_main!(benches);
