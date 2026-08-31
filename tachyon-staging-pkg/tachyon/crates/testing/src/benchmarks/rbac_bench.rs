// RBAC benchmarks - measure permission check throughput.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tachyon_rbac::types::{Action, Effect, Resource, Subject};
use tachyon_rbac::{Permission, PermissionChecker};

fn bench_permission_check_simple(c: &mut Criterion) {
    let mut checker = PermissionChecker::new();
    checker.add_permission(Permission::new("read", "document:*", "read", Effect::Allow));
    checker.add_permission(Permission::new("write", "document:*", "write", Effect::Allow));
    checker.add_permission(Permission::new("admin", "*", "*", Effect::Allow));

    let subject = Subject::new("user", "bench-user");
    let resource = Resource::new("document", "bench-doc-1");
    let action = Action::new("read");

    c.bench_function("rbac_permission_check_hit", |b| {
        b.iter(|| {
            black_box(
                checker.check_permission(
                    black_box(&subject),
                    black_box(&resource),
                    black_box(&action),
                ),
            )
        })
    });
}

fn bench_permission_check_miss(c: &mut Criterion) {
    let mut checker = PermissionChecker::new();
    checker.add_permission(Permission::new("read", "document:*", "read", Effect::Allow));

    let subject = Subject::new("user", "bench-user");
    let resource = Resource::new("document", "bench-doc-1");
    let action = Action::new("delete"); // No matching permission.

    c.bench_function("rbac_permission_check_miss", |b| {
        b.iter(|| {
            black_box(
                checker.check_permission(
                    black_box(&subject),
                    black_box(&resource),
                    black_box(&action),
                ),
            )
        })
    });
}

fn bench_permission_check_100_rules(c: &mut Criterion) {
    let mut checker = PermissionChecker::new();
    for i in 0..100 {
        checker.add_permission(
            Permission::new(
                &format!("perm-{}", i),
                &format!("resource:{}", i),
                &format!("action-{}", i),
                if i % 2 == 0 { Effect::Allow } else { Effect::Deny },
            ),
        );
    }
    // Add a catch-all at the end.
    checker.add_permission(Permission::new("catchall", "*", "*", Effect::Deny));

    let subject = Subject::new("user", "bench-user");
    let resource = Resource::new("document", "bench-doc");
    let action = Action::new("nonexistent");

    c.bench_function("rbac_permission_check_100_rules_miss", |b| {
        b.iter(|| {
            black_box(
                checker.check_permission(
                    black_box(&subject),
                    black_box(&resource),
                    black_box(&action),
                ),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_permission_check_simple,
    bench_permission_check_miss,
    bench_permission_check_100_rules,
);
criterion_main!(benches);
