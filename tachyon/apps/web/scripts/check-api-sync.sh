#!/usr/bin/env bash
# CI/local gate: the committed OpenAPI snapshot must match the server's
# authoritative spec. Server-side invariant tests (api_docs.rs) guarantee
# the spec's shape; this script guarantees the CLIENT snapshot is in sync.
#
# Usage: bash tachyon/apps/web/scripts/check-api-sync.sh
set -e
cd "$(dirname "$0")/../../.."

SPEC=/tmp/tachyon-openapi-check.json

# 1. dump the authoritative spec (pure function of server code, no server needed)
PATH="$HOME/.rustup/toolchains/1.94.1-x86_64-unknown-linux-gnu/bin:$PATH" \
  cargo test -p tachyon-server --lib api_docs::tests::dump_spec -- --ignored --nocapture 2>/dev/null \
  | sed -n '/^{/,$p' > "$SPEC"

python3 - "$SPEC" <<'EOF'
import json, sys
raw = open(sys.argv[1]).read()
d = json.loads(raw[raw.index('{'): raw.rindex('}') + 1])
# normalize: same-origin server entry (snapshot convention)
d['servers'] = [{'description': 'Tachyon API v1', 'url': '/api/v1'}]
json.dump(d, open(sys.argv[1], 'w'), indent=2)
print(f"authoritative spec: {len(d['paths'])} paths")
EOF

# 2. compare against the committed snapshot (normalized JSON equality)
python3 - "$SPEC" <<'EOF'
import json, sys
a = json.load(open(sys.argv[1]))
b = json.load(open('apps/web/src/lib/api/openapi.json'))
ka = json.dumps(a, sort_keys=True)
kb = json.dumps(b, sort_keys=True)
if ka == kb:
    print("API SYNC: OK — snapshot matches the server spec")
else:
    # structural diff detail
    def flat(spec):
        out = {}
        for p, ops in spec['paths'].items():
            for m, op in ops.items():
                out[f"{m.upper()} {p}"] = sorted(pp.get('name', '') for pp in op.get('parameters', []))
        return out
    fa, fb = flat(a), flat(b)
    only_a = sorted(set(fa) - set(fb))
    only_b = sorted(set(fb) - set(fa))
    changed = sorted(k for k in set(fa) & set(fb) if fa[k] != fb[k])
    print(f"API SYNC: DRIFT — server-only: {only_a[:6]} | client-only: {only_b[:6]} | param-changed: {changed[:6]}")
    print("fix: regenerate the server annotations or re-run the snapshot step")
    sys.exit(1)
EOF
