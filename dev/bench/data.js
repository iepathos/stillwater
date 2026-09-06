window.BENCHMARK_DATA = {
  "lastUpdate": 1788658431884,
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
          "id": "3389e164a0b54ee352df5837748c040847b3e430",
          "message": "fix(ci): Propagate security audit failures\n\nFail the audit job when cargo-audit or JSON reporting fails instead of\nsilently reporting success. Display the full report and preserve the\nraw artifact for failed runs.",
          "timestamp": "2026-09-05T20:30:27-05:00",
          "tree_id": "3503fa8962805cd23cb9f8da0e0a01019e7f3a12",
          "url": "https://github.com/iepathos/stillwater/commit/3389e164a0b54ee352df5837748c040847b3e430"
        },
        "date": 1788658430461,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 90,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "context/manual_context",
            "value": 67,
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
            "value": 0,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par_all",
            "value": 439,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 405,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/par2_heterogeneous",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "validation/stillwater_accumulate",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "validation/manual_accumulate",
            "value": 73,
            "range": "± 1",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}