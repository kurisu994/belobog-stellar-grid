window.BENCHMARK_DATA = {
  "lastUpdate": 1773994514140,
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
          "id": "94e6ea3a83d877776a0a9dcb5c9496c4c57ba88a",
          "message": "📦 dep(deps): 更新 Cargo.lock 依赖版本\n\n- 更新 `rand` 依赖版本从 1.0.13 升级至 1.0.14\n- 更新 `syn` 依赖版本从 1.2.56 升级至 1.2.57\n- 更新相应依赖包的校验和\n- 同步项目依赖至最新兼容版本",
          "timestamp": "2026-03-16T14:04:31+08:00",
          "tree_id": "cbd8c44b918c9544da1a3cf34037ad8078bf6043",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/94e6ea3a83d877776a0a9dcb5c9496c4c57ba88a"
        },
        "date": 1773641274680,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 67573,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 833246,
            "range": "± 26730",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4506853,
            "range": "± 25042",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4195256,
            "range": "± 27052",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 860641,
            "range": "± 4611",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2609145,
            "range": "± 36954",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 24212612,
            "range": "± 130323",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 125559901,
            "range": "± 739117",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 143533724,
            "range": "± 657714",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 25341046,
            "range": "± 303005",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 38,
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
          "id": "5f6e83bd5bf29fa05d8aee5cb7d244c4af58b393",
          "message": "📝 docs: 完善 Excel 在线预览开发计划与 README\n\n统一并细化 Excel 在线预览的设计与实现路线，明确选型与交付目标，便于后续开发与评审。\n\n- 明确首选 calamine（含 calamine-styles fork）并给出兜底方案\n- 补充参考实现分析、架构图、模块职责与 WASM API（HTML/JSON 双输出）\n- 规定样式映射、主题色/边框处理与安全防护策略\n- 制定测试用例、CI 要求、性能与体积优化计划\n- 同步更新 README，反映选型与功能概述\n\n破坏性变更：无",
          "timestamp": "2026-03-18T10:44:29+08:00",
          "tree_id": "78d48c77fefa05e1756d805fa0fd333c0280d4e2",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/5f6e83bd5bf29fa05d8aee5cb7d244c4af58b393"
        },
        "date": 1773802073069,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 81072,
            "range": "± 1413",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 867399,
            "range": "± 3299",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4406865,
            "range": "± 22303",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4152333,
            "range": "± 20990",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 845843,
            "range": "± 5341",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 3044003,
            "range": "± 12537",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 28266768,
            "range": "± 109968",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 156183643,
            "range": "± 702021",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 181335437,
            "range": "± 870594",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 30114036,
            "range": "± 117823",
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
            "value": 44,
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
          "id": "8a6c36a0f830f4c252a5012ddc97519f99fab9f6",
          "message": "🐛 fix(benchmarks): 修复基准测试中缺失的样式表字段\n\n修复基准测试代码中 TableData 结构体初始化时缺少 style_sheet 字段的问题\n确保测试代码与数据结构定义保持一致\n避免因字段缺失导致的编译错误或运行时问题",
          "timestamp": "2026-03-19T16:39:32+08:00",
          "tree_id": "ef1b0678c84ddc2f4c7720062ee3026b3523f66d",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/8a6c36a0f830f4c252a5012ddc97519f99fab9f6"
        },
        "date": 1773909763736,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 81833,
            "range": "± 857",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 873120,
            "range": "± 4563",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4408993,
            "range": "± 56223",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4096813,
            "range": "± 24942",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 847284,
            "range": "± 3923",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2927255,
            "range": "± 6608",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 27469883,
            "range": "± 93753",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 149373640,
            "range": "± 636866",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 173721911,
            "range": "± 458881",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 29053061,
            "range": "± 104088",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 8,
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
          "id": "3fc4a82309db7c0103c84cdd0a18f8f88fa46fe1",
          "message": "chore: bump version to 1.1.1",
          "timestamp": "2026-03-19T16:42:02+08:00",
          "tree_id": "2c47598d6f00c3fbe4949977fda5305ad433bca8",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/3fc4a82309db7c0103c84cdd0a18f8f88fa46fe1"
        },
        "date": 1773909913447,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 82349,
            "range": "± 648",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 863582,
            "range": "± 16061",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4501322,
            "range": "± 12534",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4299115,
            "range": "± 28826",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 892870,
            "range": "± 6509",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2979962,
            "range": "± 24276",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 27966982,
            "range": "± 93400",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 152834744,
            "range": "± 319541",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 177526266,
            "range": "± 612827",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 29884168,
            "range": "± 143317",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 46,
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
          "id": "91f7fc8ffd991a188ce43ff88614a7079d26ffc4",
          "message": "✨ feat(Excel预览): 添加Excel文件解析与HTML表格生成\n- 实现ParsedSheet到HTML表格的转换\n- 添加XSS防护功能，确保内容安全\n- 支持合并单元格和样式应用\n- 增加相关单元测试，确保功能正确性",
          "timestamp": "2026-03-20T11:50:59+08:00",
          "tree_id": "1d75b80e08123f0b1063db40a1594960c9c993a7",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/91f7fc8ffd991a188ce43ff88614a7079d26ffc4"
        },
        "date": 1773978877094,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 83916,
            "range": "± 4400",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 858282,
            "range": "± 5298",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4585913,
            "range": "± 25383",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4308291,
            "range": "± 23055",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 887757,
            "range": "± 1799",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2000843,
            "range": "± 8094",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 17795017,
            "range": "± 59030",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 98138675,
            "range": "± 196653",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 106507296,
            "range": "± 383234",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 18736724,
            "range": "± 85527",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 54,
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
          "id": "7f324657d7c514bf6aa51bf20899416f8c67bbdc",
          "message": "✨ feat(远程文件加载): 添加从URL加载远程Excel文件功能\n- 实现loadUrl方法以支持远程文件加载\n- 添加URL输入区域和相关样式\n- 处理加载过程中的错误提示",
          "timestamp": "2026-03-20T12:04:13+08:00",
          "tree_id": "8ce6f0f0e4e6abe1fafddf57cec287dcdc38558b",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/7f324657d7c514bf6aa51bf20899416f8c67bbdc"
        },
        "date": 1773979655769,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 95821,
            "range": "± 2659",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 991497,
            "range": "± 17892",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 5150978,
            "range": "± 11208",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4845281,
            "range": "± 11905",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 1020594,
            "range": "± 8602",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2001135,
            "range": "± 27985",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 17742118,
            "range": "± 155180",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 98610944,
            "range": "± 500442",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 106149678,
            "range": "± 517517",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 18667788,
            "range": "± 58930",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 54,
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
          "id": "60678051cd62bb0b09825fbf1f4618d1702a933c",
          "message": "🐛 fix(测试): 修复分块CSV写入测试中的变量声明\n- 将所有片段的声明从可变改为不可变",
          "timestamp": "2026-03-20T14:04:40+08:00",
          "tree_id": "19e3eee44ac91032eaac24a97df4e27aee1961b7",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/60678051cd62bb0b09825fbf1f4618d1702a933c"
        },
        "date": 1773986892789,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 91662,
            "range": "± 727",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 958345,
            "range": "± 4374",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4875553,
            "range": "± 11642",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4677259,
            "range": "± 105357",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 965875,
            "range": "± 26275",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2084179,
            "range": "± 8301",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 18124067,
            "range": "± 38674",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 101286356,
            "range": "± 159684",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 111773556,
            "range": "± 93476",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 19058359,
            "range": "± 50433",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 46,
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
          "id": "f77cbf4938a7007dd6c6e22a8047e6a6ebf7f2d5",
          "message": "chore: bump version to 1.1.2",
          "timestamp": "2026-03-20T14:05:09+08:00",
          "tree_id": "f4c871cbe332a3e441c55ae694b4ced82c16fd4f",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/f77cbf4938a7007dd6c6e22a8047e6a6ebf7f2d5"
        },
        "date": 1773986911450,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 84516,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 902838,
            "range": "± 14543",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4584403,
            "range": "± 40968",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4316011,
            "range": "± 24520",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 890794,
            "range": "± 6217",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 1999364,
            "range": "± 6266",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 18454393,
            "range": "± 78961",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 102042577,
            "range": "± 3255786",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 111262422,
            "range": "± 939026",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 19437232,
            "range": "± 59067",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 54,
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
          "id": "1bda9bc7e48d25a86c503cc524d3f10e0de9a793",
          "message": "🔧 chore(workflow): 同步子包依赖版本并更新发布脚本\n\n- 在 `bump-core` 脚本中添加自动化步骤，用于同步所有子包（React、Vue、Svelte、Solid）中对 `@bsg-export/types` 的依赖版本\n- 将子包 `package.json` 中的依赖声明从本地文件引用（`file:../types`）更新为具体的语义化版本号（`^1.1.2`）\n- 确保在发布新版本时，所有相关包的依赖版本保持同步和一致\n- 改进发布流程的健壮性，减少因依赖版本不一致导致的问题",
          "timestamp": "2026-03-20T14:30:20+08:00",
          "tree_id": "71b4ce29f9bb6aa7ceb2d92e6d54ec4adc2e0568",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/1bda9bc7e48d25a86c503cc524d3f10e0de9a793"
        },
        "date": 1773988418081,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 83144,
            "range": "± 1494",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 858497,
            "range": "± 2533",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4576915,
            "range": "± 34368",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4212267,
            "range": "± 93483",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 889948,
            "range": "± 6905",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 1978058,
            "range": "± 6452",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 18219761,
            "range": "± 59588",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 100601738,
            "range": "± 254352",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 108372451,
            "range": "± 745332",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 19244964,
            "range": "± 74698",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 53,
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
          "id": "fff53a8ceb25b35d1332ded2ee83ec7d7543d120",
          "message": "chore: bump version to 1.1.3",
          "timestamp": "2026-03-20T14:30:45+08:00",
          "tree_id": "d76147476af62d82cd217598b8fe3e1060ed409a",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/fff53a8ceb25b35d1332ded2ee83ec7d7543d120"
        },
        "date": 1773988446616,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 84684,
            "range": "± 2884",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 885696,
            "range": "± 6938",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4561009,
            "range": "± 11920",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4318706,
            "range": "± 55883",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 893626,
            "range": "± 4173",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 1972751,
            "range": "± 5615",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 18326911,
            "range": "± 96042",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 101067149,
            "range": "± 400622",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 109725339,
            "range": "± 2113432",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 19294299,
            "range": "± 137139",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 47,
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
          "id": "4a4319edf7ea56d139a81ec670e297f003902537",
          "message": "🐛 fix(预览): 优化表格渲染样式\n\n- 仅在填充模式为 solid 时应用背景色，避免误渲染其他图案填充\n- 为预览表格注入独立样式并改用类名，提升样式隔离和兼容性\n- 将列宽改为最小宽度，允许内容按需扩展，改善显示效果\n- 调整自动换行与细边框表现，提升长文本和细线条的可读性\n- 收紧单元格宽度估算，避免预览过宽或过窄",
          "timestamp": "2026-03-20T15:46:36+08:00",
          "tree_id": "e4869b831656ace01ddc42411f668593ba858c10",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/4a4319edf7ea56d139a81ec670e297f003902537"
        },
        "date": 1773993001113,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 68123,
            "range": "± 2172",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 898685,
            "range": "± 2228",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4597791,
            "range": "± 15476",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4270604,
            "range": "± 17722",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 855444,
            "range": "± 2827",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 1820017,
            "range": "± 5811",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 17443043,
            "range": "± 63338",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 91722080,
            "range": "± 407170",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 95562577,
            "range": "± 319240",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 18067714,
            "range": "± 87804",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 7,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 41,
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
          "id": "798e8565af7748cb410c8a88f79c0b76c1d243ac",
          "message": "✨ feat(预览): 支持隐藏表过滤与格式精显\n\n- 自动识别隐藏/极度隐藏工作表，并默认跳过\n- 切换工作表时按可见列表映射原始索引\n- 按 Excel 格式动态显示小数、百分比和千分位\n- 优化预览样式，使显示更接近 Excel\n- 补充文档、示例与测试说明",
          "timestamp": "2026-03-20T16:11:46+08:00",
          "tree_id": "688b398ebf382412eebfd4ec59c55e00ae6e365b",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/798e8565af7748cb410c8a88f79c0b76c1d243ac"
        },
        "date": 1773994512884,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 82556,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 890118,
            "range": "± 12902",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4497243,
            "range": "± 33854",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4236022,
            "range": "± 45423",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 859342,
            "range": "± 14854",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 1983301,
            "range": "± 7052",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 18195711,
            "range": "± 153830",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 99589181,
            "range": "± 1487179",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 107277601,
            "range": "± 436774",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 19206892,
            "range": "± 236090",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 9,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 47,
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
          "id": "c8b961cf25c805a40cfd468a6fb6cf73ec64d05a",
          "message": "chore: bump version to 1.1.4",
          "timestamp": "2026-03-20T16:12:01+08:00",
          "tree_id": "968e59e6fb94660082bbb60f04f3d38908ec6adb",
          "url": "https://github.com/kurisu994/belobog-stellar-grid/commit/c8b961cf25c805a40cfd468a6fb6cf73ec64d05a"
        },
        "date": 1773994513608,
        "tool": "cargo",
        "benches": [
          {
            "name": "csv_generation/无BOM/100行x10列",
            "value": 81682,
            "range": "± 1629",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x10列",
            "value": 848942,
            "range": "± 14131",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/5000行x10列",
            "value": 4438903,
            "range": "± 36702",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/无BOM/1000行x50列",
            "value": 4151922,
            "range": "± 49148",
            "unit": "ns/iter"
          },
          {
            "name": "csv_generation/带BOM/1000行x10列",
            "value": 872300,
            "range": "± 5815",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/100行x10列",
            "value": 2018081,
            "range": "± 15449",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x10列",
            "value": 18452228,
            "range": "± 92504",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/5000行x10列",
            "value": 101892330,
            "range": "± 553725",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/无合并/1000行x50列",
            "value": 109856435,
            "range": "± 1039838",
            "unit": "ns/iter"
          },
          {
            "name": "xlsx_generation/含合并/1000行x10列",
            "value": 19117305,
            "range": "± 111270",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/普通文本",
            "value": 8,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "csv_escape/公式文本",
            "value": 47,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}