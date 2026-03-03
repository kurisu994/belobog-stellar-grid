window.BENCHMARK_DATA = {
  "lastUpdate": 1772524946521,
  "repoUrl": "https://github.com/kurisu994/belobog-stellar-grid",
  "entries": {
    "Rust Benchmark": [
      {
        "commit": {
          "author": {
            "email": "makise_kurisuu@outlook.jp",
            "name": "Kurisu",
            "username": "kurisu994"
          },
          "committer": {
            "email": "makise_kurisuu@outlook.jp",
            "name": "Kurisu",
            "username": "kurisu994"
          },
          "distinct": true,
          "id": "2bd977082decb21f6aff9788b5dffe564704198e",
          "message": "🔧 chore(CI): 修复基准测试工作流中的分支切换\n\n将 `git checkout -` 改为显式切换回 `main` 分支\n- 确保工作流在创建 gh-pages 分支后能正确返回主分支\n- 避免因符号引用可能导致的意外行为",
          "timestamp": "2026-03-03T15:58:49+08:00",
          "tree_id": "7ec37d868792f59571d95071b9a9805eebd81d5a",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/2bd977082decb21f6aff9788b5dffe564704198e"
        },
        "date": 1772524945699,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 78119,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 845481,
            "range": "± 4110",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4311479,
            "range": "± 28646",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4025617,
            "range": "± 18642",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 846502,
            "range": "± 10675",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2960997,
            "range": "± 28528",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 27695269,
            "range": "± 129120",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 150879131,
            "range": "± 545754",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 177986595,
            "range": "± 1387897",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 29496519,
            "range": "± 186031",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 6,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 45,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}