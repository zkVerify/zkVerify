# Kimchi verifier

The Kimchi verifier accepts a consensus-defined compatibility profile rather
than arbitrary Kimchi parameters. The production profile is `Vesta16`.

## Production compatibility profile

`Vesta16` currently fixes the following verification envelope:

| Parameter | Value |
| --- | --- |
| Curve | Vesta |
| IPA SRS size / maximum polynomial size | `2^16` |
| Supported evaluation domains | `2^3` through `2^18` |
| Maximum commitment chunks | `4` |
| Maximum public inputs | `1024` |
| Recursive previous challenges | Unsupported |

The SRS, supported domain/chunk envelope, and authenticated parameter digests
are shared by the runtime verifier and the native Vesta host functions.

## Native compatibility and upgrades

The existing `Vesta16` parameters are a native node compatibility boundary and
must not be changed in place. A different SRS size, curve, domain/chunk
envelope, or parameter set cannot be supported through a runtime-only upgrade.

Supporting such a change requires:

1. re-evaluating the native design and introducing a distinct, versioned
   verifier/profile path;
2. adding or versioning the required host functions while retaining those
   needed by active runtimes;
3. updating the native parameters, authenticated digests, and cache identity;
4. regenerating fixtures, benchmarks, and weights and testing both
   compatibility paths;
5. coordinating a compatible node rollout before activating runtime support.

Upstream expectations about future SRS sizes are planning input, not a
permanent protocol guarantee. Reconfirm them before integrating future Kimchi
or o1js hard-fork changes.
