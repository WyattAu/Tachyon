use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tachyon_core::id::{SessionId, UserId};
use tachyon_rbac::types::{AccessRequest, Action, AuthContext, Resource, Subject};
use tachyon_rbac::Enforcer;

fn bench_authorize(c: &mut Criterion) {
    let mut group = c.benchmark_group("rbac_authorize");

    for policies in [1, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("policies", policies),
            &policies,
            |b, &num_policies| {
                b.iter(|| {
                    let mut enforcer = Enforcer::new();

                    for i in 0..num_policies {
                        let policy = tachyon_rbac::Policy::new(
                            format!("bench-policy-{}", i),
                            format!("Bench Policy {}", i),
                            tachyon_rbac::PolicyType::Rbac,
                        )
                        .add_rule(tachyon_rbac::PolicyRule::new(
                            format!("bench-rule-{}", i),
                            "role:admin",
                            "document:*",
                            "read",
                            tachyon_rbac::types::Effect::Allow,
                        ));
                        enforcer.policy_engine().add_policy(policy);
                    }

                    let subject = Subject::new("user", "benchmark-user");
                    let resource = Resource::new("document", "benchmark-doc");
                    let action = Action::new("read");
                    let context =
                        AuthContext::new(UserId::new(), SessionId::new()).with_role("admin");
                    let request = AccessRequest::new(
                        black_box(subject),
                        black_box(resource),
                        black_box(action),
                        black_box(context),
                    );

                    let _ = black_box(enforcer.authorize(&request));
                });
            },
        );
    }
    group.finish();
}

fn bench_authorize_cached(c: &mut Criterion) {
    let mut group = c.benchmark_group("rbac_authorize_cached");

    let mut enforcer = Enforcer::new();
    let policy = tachyon_rbac::Policy::new("cached-policy", "Cached Policy", tachyon_rbac::PolicyType::Rbac)
        .add_rule(tachyon_rbac::PolicyRule::new(
            "cached-rule",
            "user:*",
            "document:*",
            "read",
            tachyon_rbac::types::Effect::Allow,
        ));
    enforcer.policy_engine().add_policy(policy);

    let subject = Subject::new("user", "cached-user");
    let resource = Resource::new("document", "cached-doc");
    let action = Action::new("read");
    let context = AuthContext::new(UserId::new(), SessionId::new());
    let request = AccessRequest::new(subject, resource, action, context);

    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            let _ = black_box(enforcer.authorize(&request));
        });
    });

    group.finish();
}

criterion_group!(benches, bench_authorize, bench_authorize_cached);
criterion_main!(benches);
