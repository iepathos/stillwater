window.BENCHMARK_DATA = {
  "lastUpdate": 1779519437466,
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
          "id": "412729a1483c6f4f3ad23a65413b45b9a3ce1c48",
          "message": "fix(ci): Store cargo benchmark output",
          "timestamp": "2026-05-22T23:01:46-07:00",
          "tree_id": "74553f0551dfbf0a3ad15dddc2b21c58179636f7",
          "url": "https://github.com/iepathos/stillwater/commit/412729a1483c6f4f3ad23a65413b45b9a3ce1c48"
        },
        "date": 1779516266528,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 126,
            "range": "± 2",
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
            "value": 433,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 342,
            "range": "± 1",
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
            "value": 90,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "validation/manual_accumulate",
            "value": 61,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "5e726bff4e8dea2ee96db40f85ff3e578a142326",
          "message": "chore(bench): Format benchmark sources",
          "timestamp": "2026-05-22T23:14:17-07:00",
          "tree_id": "cbcd3721a5e646537b489e9a42d6e399b3531cfc",
          "url": "https://github.com/iepathos/stillwater/commit/5e726bff4e8dea2ee96db40f85ff3e578a142326"
        },
        "date": 1779517021066,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "context/manual_context",
            "value": 73,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "effects/stillwater_chain",
            "value": 35,
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
            "value": 445,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 350,
            "range": "± 6",
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
            "value": 83,
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
          "id": "b1a7741e33aa03d9d796bbfc3f6b052a77ba8f6d",
          "message": "chore: Add commit quality tooling",
          "timestamp": "2026-05-22T23:16:58-07:00",
          "tree_id": "d8300cb6cc1f03d286042b4672b268d5886c1d5d",
          "url": "https://github.com/iepathos/stillwater/commit/b1a7741e33aa03d9d796bbfc3f6b052a77ba8f6d"
        },
        "date": 1779517275861,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 102,
            "range": "± 17",
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
            "value": 45,
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
            "value": 435,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 350,
            "range": "± 9",
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
            "value": 83,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "validation/manual_accumulate",
            "value": 58,
            "range": "± 1",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "1e256175de1f67c3b3380b4bdd090482977a486a",
          "message": "chore(release): Prepare 1.1.0",
          "timestamp": "2026-05-22T23:32:56-07:00",
          "tree_id": "ebe88a3e842ab4f1d0532bbc64cea6a153f212c5",
          "url": "https://github.com/iepathos/stillwater/commit/1e256175de1f67c3b3380b4bdd090482977a486a"
        },
        "date": 1779519436305,
        "tool": "cargo",
        "benches": [
          {
            "name": "context/stillwater_context",
            "value": 135,
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
            "value": 35,
            "range": "± 1",
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
            "value": 478,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parallel/sequential",
            "value": 361,
            "range": "± 7",
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
            "value": 83,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "validation/manual_accumulate",
            "value": 58,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}