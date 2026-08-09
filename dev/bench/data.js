window.BENCHMARK_DATA = {
  "lastUpdate": 1786316785846,
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
      }
    ]
  }
}