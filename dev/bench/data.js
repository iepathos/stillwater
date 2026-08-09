window.BENCHMARK_DATA = {
  "lastUpdate": 1786316819327,
  "repoUrl": "https://github.com/iepathos/stillwater",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "875fc5063505973e2fff1e4990687ede26a9228a",
          "message": "deps(deps): bump tokio from 1.52.3 to 1.53.1 (#29)\n\nBumps [tokio](https://github.com/tokio-rs/tokio) from 1.52.3 to 1.53.1.\n- [Release notes](https://github.com/tokio-rs/tokio/releases)\n- [Commits](https://github.com/tokio-rs/tokio/compare/tokio-1.52.3...tokio-1.53.1)\n\n---\nupdated-dependencies:\n- dependency-name: tokio\n  dependency-version: 1.53.1\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-09T16:03:19-07:00",
          "tree_id": "2f486059af1330dc535c3ef1a2dff9f5c5c77edf",
          "url": "https://github.com/iepathos/stillwater/commit/875fc5063505973e2fff1e4990687ede26a9228a"
        },
        "date": 1786316784640,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 126,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/manual_context",
            "value": 76,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "effects/stillwater_chain",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "effects/manual_chain",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par_all",
            "value": 447,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 342,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par2_heterogeneous",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "validation/stillwater_accumulate",
            "value": 88,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "validation/manual_accumulate",
            "value": 60,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "699604d53dcbdb52d19daf0f84cb0a21777f32ca",
          "message": "deps(deps): bump serde_json from 1.0.150 to 1.0.151 (#28)\n\nBumps [serde_json](https://github.com/serde-rs/json) from 1.0.150 to 1.0.151.\n- [Release notes](https://github.com/serde-rs/json/releases)\n- [Commits](https://github.com/serde-rs/json/compare/v1.0.150...v1.0.151)\n\n---\nupdated-dependencies:\n- dependency-name: serde_json\n  dependency-version: 1.0.151\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-09T16:03:30-07:00",
          "tree_id": "2d782b275a12a1e28f686c174faf73b141105886",
          "url": "https://github.com/iepathos/stillwater/commit/699604d53dcbdb52d19daf0f84cb0a21777f32ca"
        },
        "date": 1786316793259,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 102,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/manual_context",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "effects/stillwater_chain",
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "effects/manual_chain",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par_all",
            "value": 426,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 369,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par2_heterogeneous",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "validation/stillwater_accumulate",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "validation/manual_accumulate",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3cdf72803a0bc8b7e6f149376cce053daef7ae95",
          "message": "deps(deps): bump serde from 1.0.228 to 1.0.229 (#26)\n\nBumps [serde](https://github.com/serde-rs/serde) from 1.0.228 to 1.0.229.\n- [Release notes](https://github.com/serde-rs/serde/releases)\n- [Commits](https://github.com/serde-rs/serde/compare/v1.0.228...v1.0.229)\n\n---\nupdated-dependencies:\n- dependency-name: serde\n  dependency-version: 1.0.229\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2026-08-09T16:03:41-07:00",
          "tree_id": "e638fa9be8e39800249be57a137395ac89f510ac",
          "url": "https://github.com/iepathos/stillwater/commit/3cdf72803a0bc8b7e6f149376cce053daef7ae95"
        },
        "date": 1786316818656,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/manual_context",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "effects/stillwater_chain",
            "value": 48,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "effects/manual_chain",
            "value": 1,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par_all",
            "value": 428,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 362,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par2_heterogeneous",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "validation/stillwater_accumulate",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "validation/manual_accumulate",
            "value": 61,
            "range": "± 1",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}