# 默认命令：显示帮助
default:
    @just --list

# -----------------------------------------------------------------------------
# 环境检查
# -----------------------------------------------------------------------------

# 检查必要的开发工具
check-tools:
    @echo "🔍 检查开发环境依赖..."
    @if ! command -v wasm-pack &> /dev/null; then echo "❌ 未找到 wasm-pack (请运行 cargo install wasm-pack)"; exit 1; fi
    @if ! command -v basic-http-server &> /dev/null; then echo "❌ 未找到 basic-http-server (请运行 cargo install basic-http-server)"; exit 1; fi
    @if ! command -v wasm-opt &> /dev/null; then echo "⚠️  未找到 wasm-opt (建议安装 binaryen 以优化构建体积)"; fi
    @# 检查 cargo-set-version (cargo-edit 的一部分)
    @if ! cargo set-version --version &> /dev/null; then echo "❌ 未找到 cargo-set-version (请运行 cargo install cargo-edit)"; exit 1; fi
    @echo "✅ 开发环境检查通过"

# -----------------------------------------------------------------------------
# 开发与构建
# -----------------------------------------------------------------------------

# 启动开发模式 (构建并启动服务器)
dev: check-tools build
    @echo "🚀 启动本地服务器..."
    basic-http-server .

# 构建 WASM
build:
    @echo "🔨 构建 WebAssembly..."
    wasm-pack build --target web --out-dir pkg

# 优化 WASM (需要 wasm-opt)
optimize:
    #!/bin/bash
    if command -v wasm-opt &> /dev/null; then
        echo "⚡ 优化 WASM 文件..."
        wasm-opt -Oz pkg/belobog_stellar_grid_bg.wasm -o pkg/belobog_stellar_grid_bg.wasm
    else
        echo "⚠️  wasm-opt 未安装，跳过优化"
    fi

# -----------------------------------------------------------------------------
# 代码质量
# -----------------------------------------------------------------------------

# 代码格式化
fmt:
    @echo "🎨 格式化代码..."
    cargo fmt

# 代码质量检查
lint:
    @echo "🔍 运行 Clippy 代码质量检查..."
    cargo clippy -- -D warnings

# 全面检查 (格式化 + Lint)
check: fmt lint
    @echo "✅ 代码检查和格式化完成"

# 运行测试
test:
    @echo "🧪 运行测试..."
    cargo test

# 运行 E2E 测试 (Playwright)
e2e: build
    @echo "🌐 运行 E2E 测试..."
    cd e2e && npx playwright test

# 运行 E2E 测试 (带浏览器界面)
e2e-headed: build
    @echo "🌐 运行 E2E 测试 (headed)..."
    cd e2e && npx playwright test --headed

# -----------------------------------------------------------------------------
# 发布流程
# -----------------------------------------------------------------------------

# 内部配方：升级版本 (逻辑核心)
bump-core level:
    #!/bin/bash
    set -e
    # 获取当前版本
    current=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "📌 当前版本: $current"
    echo "🔖 升级级别: {{level}}"
    
    # 升级 Cargo.toml
    cargo set-version --bump {{level}}
    new=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "✅ Cargo.toml 版本已更新: $current -> $new"
    
    # 更新 README.md
    # 使用 perl 来处理跨平台兼容性 (macOS/Linux)
    if perl -i -pe "s/version-$current/version-$new/g" README.md; then
        echo "✅ README.md 版本已更新: $current -> $new"
    else
        echo "⚠️  README.md 版本更新失败 (可能未找到匹配版本号)"
    fi
    
    # 同步子包版本
    for pkg in packages/types packages/react packages/vue; do
        if [ -f "$pkg/package.json" ]; then
            perl -i -pe "s/\"version\": \".*?\"/\"version\": \"$new\"/" "$pkg/package.json"
            echo "✅ $pkg/package.json 版本已更新: -> $new"
        fi
    done
    
    # 更新 CHANGELOG.md：将 [Unreleased] 替换为版本号 + 日期
    if grep -q '## \[Unreleased\]' CHANGELOG.md; then
        today=$(date +%Y-%m-%d)
        perl -i -pe "s/## \\[Unreleased\\]/## \\[Unreleased\\]\\n\\n---\\n\\n## [$new] - $today/" CHANGELOG.md
        echo "✅ CHANGELOG.md 已更新: [Unreleased] -> [$new] - $today"
    fi

# 升级版本 (手动模式)
bump level: (bump-core level)
    @echo ""
    @echo "✅ 版本升级完成。请检查更改并提交。"

# 发布并升级版本 (本地完整流程)
# 用法: just release patch
release level: check test (bump-core level)
    @echo ""
    @echo "🎉 准备发布新版本..."
    @echo "请运行 'just publish' 将构建产物发布到 npm"

# CI 自动发布 (GitHub Actions 使用)
# 用法: just ci-release patch  # 或 minor/major
ci-release level: check test
    #!/bin/bash
    set -e
    
    # 检查工作区是否干净
    if ! git diff --quiet; then
        echo "⚠️  工作区有未提交的更改，请先处理"
        git status --short
        exit 1
    fi

    # 调用核心版本升级逻辑
    just bump-core {{level}}
    
    # 获取新版本号
    new=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    
    echo ""
    echo "📋 将要执行的操作:"
    echo "   1. git add ."
    echo "   2. git commit -m \"chore: bump version to $new\""
    echo "   3. git tag v$new"
    echo "   4. git push origin main --tags"
    echo ""
    
    # 交互式确认 (仅在非 CI 环境或显式交互时)
    # 注意: 在 CI 环境中通常会自动同意，但在本地运行 ci-release 时需要确认
    read -p "确认发布 v$new 到 npm? (y/N) " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ 已取消，回滚版本..."
        git checkout Cargo.toml README.md
        exit 1
    fi
    
    # 提交和推送
    git add .
    git commit -m "chore: bump version to $new"
    git tag "v$new"
    git push origin main --tags
    
    echo ""
    echo "🎉 已推送 v$new，GitHub Actions 将自动发布到 npm"
    echo "📦 查看进度: https://github.com/kurisu994/belobog-stellar-grid/actions"

# -----------------------------------------------------------------------------
# 子包构建与发布
# -----------------------------------------------------------------------------

# 构建子包 (types/react/vue)
build-packages:
    #!/bin/bash
    set -e
    for pkg in packages/types packages/react packages/vue; do
        if [ -d "$pkg" ]; then
            echo "📦 构建 $pkg..."
            cd "$pkg" && pnpm install && pnpm run build && cd ../.. 
            echo "✅ $pkg 构建完成"
        fi
    done

# 发布子包到 npm
publish-packages tag="latest":
    #!/bin/bash
    set -e
    tag="{{tag}}"
    for pkg in packages/types packages/react packages/vue; do
        if [ -d "$pkg" ]; then
            echo "📤 发布 $pkg (tag: $tag)..."
            cd "$pkg" && npm publish --access public --tag "$tag" && cd ../.. 
            echo "✅ $pkg 发布完成"
        fi
    done

# -----------------------------------------------------------------------------
# npm 发布
# -----------------------------------------------------------------------------

# 显示发布信息
info:
    @echo "📦 发布信息:"
    @echo "   包名: belobog-stellar-grid"
    @grep "^version" Cargo.toml | sed 's/version = /   版本: /'

# 发布前测试 (dry-run)
dry-run: build
    #!/bin/bash
    set -e
    echo "📤 运行发布前测试 (dry-run)..."
    cd pkg && npm publish --dry-run --registry https://registry.npmjs.org/
    echo "✅ dry-run 测试通过"

# 发布到 npm (带 tag)
publish tag="latest": build dry-run
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