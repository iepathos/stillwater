window.BENCHMARK_DATA = {
  "lastUpdate": 1788731888711,
  "repoUrl": "https://github.com/iepathos/stillwater",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "iepathos@gmail.com",
            "name": "Glen Baker",
            "username": "iepathos"
          },
          "committer": {
            "email": "iepathos@gmail.com",
            "name": "Glen Baker",
            "username": "iepathos"
          },
          "distinct": true,
          "id": "0761261ac301ffdfba94c4e019bec7f0a2f87c2b",
          "message": "fix(docs): Preserve the quickstart toolchain\n\nResolve the active Rust toolchain in the checkout and carry it into\nthe temporary project so directory changes cannot select a different\ncompiler. Report the toolchain and fail if resolution fails.\n\nTest toolchain propagation, exact snippets, explicit overrides, and\nfailure handling.",
          "timestamp": "2026-09-06T16:54:26-05:00",
          "tree_id": "1728905481c5d8623e73f81845891f552960a523",
          "url": "https://github.com/iepathos/stillwater/commit/0761261ac301ffdfba94c4e019bec7f0a2f87c2b"
        },
        "date": 1788731886718,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 80,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/manual_context",
            "value": 60,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "effects/stillwater_chain",
            "value": 28,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "effects/manual_chain",
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par_all",
            "value": 419,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 362,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par2_heterogeneous",
            "value": 16,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "validation/stillwater_accumulate",
            "value": 66,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "validation/manual_accumulate",
            "value": 65,
            "range": "± 2",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}