# 发布 npm 包

# 升级版本 (patch/minor/major)
bump level:
    @#!/bin/bash
    current=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "📌 当前版本: $current"
    echo "🔖 升级级别: {{level}}"
    cargo bump {{level}}
    new=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "✅ 版本已更新: $current -> $new"
    echo ""
    echo "请检查并提交更改后再次运行 just publish"

# 升级版本并发布
release level:
    @#!/bin/bash
    current=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "📌 当前版本: $current"
    echo "🔖 升级级别: {{level}}"
    cargo bump {{level}}
    new=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "✅ 版本已更新: $current -> $new"

# 运行测试
test:
    @echo "🧪 运行测试..."
    cargo test

# 构建 WASM
build:
    @echo "🔨 构建 WebAssembly..."
    wasm-pack build --target web --out-dir pkg

# 优化 WASM
optimize:
    #!/bin/bash
    if command -v wasm-opt &> /dev/null; then
        echo "⚡ 优化 WASM 文件..."
        wasm-opt -Oz pkg/belobog_stellar_grid_bg.wasm -o pkg/belobog_stellar_grid_bg.wasm
    else
        echo "⚠️  wasm-opt 未安装，跳过优化"
    fi

# 显示发布信息
info:
    @echo "📦 发布信息:"
    @echo "   包名: belobog-stellar-grid"
    @grep "^version" Cargo.toml | sed 's/version = /   版本: /'

# 发布前测试 (dry-run)
dry-run:
    #!/bin/bash
    set -e
    echo "📤 运行发布前测试 (dry-run)..."
    cd pkg && npm publish --dry-run --registry https://registry.npmjs.org/
    echo "✅ dry-run 测试通过"

# 发布到 npm (带 tag)
publish tag:
    #!/bin/bash
    set -e
    tag="{{tag}}"
    echo ""
    echo "⚠️  即将发布到 npm"
    echo "   Registry: https://registry.npmjs.org/"
    echo "   Tag: $tag"
    read -p "确认发布? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cd pkg && npm publish --registry https://registry.npmjs.org/ --tag "$tag"
        echo ""
        echo "✅ 发布成功!"
    else
        echo "❌ 取消发布"
        exit 1
    fi