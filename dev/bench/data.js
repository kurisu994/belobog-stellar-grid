window.BENCHMARK_DATA = {
  "lastUpdate": 1773371688506,
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
      },
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
          "id": "d7b135a91196648f06e0008b69f5054b8f76cb36",
          "message": "🚀 feat(CI/CD): 添加示例页面自动部署\n\n- 新增 GitHub Actions 工作流，在推送到主分支时自动构建 WASM 并部署示例页面到 GitHub Pages\n- 在 README 中添加在线演示和历史性能趋势图表的链接\n- 部署内容包括示例页面、WASM 产物和一个自动跳转的首页",
          "timestamp": "2026-03-03T16:10:48+08:00",
          "tree_id": "051fe1049b28d7b91cf220d853648fb0d8fe8931",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/d7b135a91196648f06e0008b69f5054b8f76cb36"
        },
        "date": 1772525675792,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 79012,
            "range": "± 367",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 823895,
            "range": "± 10759",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4296978,
            "range": "± 19307",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 3999077,
            "range": "± 17343",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 842424,
            "range": "± 5527",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2946174,
            "range": "± 41742",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 27503753,
            "range": "± 536578",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 148343755,
            "range": "± 394788",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 172154034,
            "range": "± 570626",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 29070039,
            "range": "± 149159",
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
      },
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
          "id": "feb8938b751189e5624808d505fb59b9027bb9dc",
          "message": "🎨 style(examples): 全面升级 CDN 导出示例页面的视觉设计和交互体验\n\n重构示例页面的整体布局和样式，移除对第三方 CSS 框架的依赖，采用自定义的现代化设计语言\n增强视觉层次和可读性，优化按钮组、表格和代码块的展示效果\n改进状态提示面板和加载动画，提升用户交互反馈的清晰度\n保持原有功能逻辑不变，专注于提升演示页面的专业性和用户体验",
          "timestamp": "2026-03-03T16:26:01+08:00",
          "tree_id": "b921cc807d27b1580e3f38a12bee92b7764a79bb",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/feb8938b751189e5624808d505fb59b9027bb9dc"
        },
        "date": 1772526565928,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 78505,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 820234,
            "range": "± 5060",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4308348,
            "range": "± 29698",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4065925,
            "range": "± 11400",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 846946,
            "range": "± 12813",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2949762,
            "range": "± 22110",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 27610081,
            "range": "± 204860",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 150231555,
            "range": "± 841381",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 174297419,
            "range": "± 1396865",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 29362137,
            "range": "± 255199",
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
      },
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
          "id": "2c7a41ba6bd4af7fabcd54ee9b11846890c575ec",
          "message": "📝 docs(plan): 添加 Excel 在线预览功能开发计划\n\n- 新增 EXCEL_PREVIEW_PLAN.md 详细规划 Rust WASM 实现方案\n- 将 Excel 在线预览功能加入 README.md 的待办事项列表\n- 规划分为四个阶段：技术预研、核心解析、前端集成、性能测试\n- 明确功能需求：保持原始样式、只渲染数据区域、只读查看",
          "timestamp": "2026-03-12T14:52:29+08:00",
          "tree_id": "3205fade9799262178d237b2b584af166a235cb0",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/2c7a41ba6bd4af7fabcd54ee9b11846890c575ec"
        },
        "date": 1773298567802,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 80977,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 853479,
            "range": "± 1886",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4353504,
            "range": "± 27498",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4041956,
            "range": "± 7245",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 850237,
            "range": "± 4780",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2964316,
            "range": "± 5494",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 27855002,
            "range": "± 93611",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 150206095,
            "range": "± 485721",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 173938377,
            "range": "± 593312",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 29528782,
            "range": "± 108421",
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
            "value": 43,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "4230eb05341fb481622705343778ab1cc6d7c988",
          "message": "🚚 chore(*): 更新依赖",
          "timestamp": "2026-03-13T11:11:00+08:00",
          "tree_id": "053c574182296ca5cec9d7e77218a7932c60cc35",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/4230eb05341fb481622705343778ab1cc6d7c988"
        },
        "date": 1773371687412,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 80802,
            "range": "± 725",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 833632,
            "range": "± 3637",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4370151,
            "range": "± 22512",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4105391,
            "range": "± 25151",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 866154,
            "range": "± 2687",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2928910,
            "range": "± 11095",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 27408026,
            "range": "± 126839",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 149489205,
            "range": "± 638814",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 174881785,
            "range": "± 923015",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 29217164,
            "range": "± 142215",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 5,
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