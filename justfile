# 默认命令：显示帮助
default:
    @just --list

# 启动开发模式 (构建并启动服务器)
dev: build
    @echo "🚀 启动本地服务器..."
    basic-http-server .
# 发布 npm 包
# 升级版本 (patch/minor/major)
bump level:
    @#!/bin/bash
    current=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "📌 当前版本: $current"
    echo "🔖 升级级别: {{level}}"
    cargo set-version --bump {{level}}
    new=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "✅ Cargo.toml 版本已更新: $current -> $new"
    if sed -i '' "s/version-$current/version-$new/g" README.md; then
    echo "✅ README.md 版本已更新: $current -> $new"
    fi
    echo ""
    echo "请检查并提交更改后再次运行 just publish"

# 🚀 一键发布到 npm（通过 GitHub Actions）
# 用法: just ci-release patch   # 或 minor/major
ci-release level:
    #!/bin/bash
    set -e
    
    # 检查工作区是否干净
    if ! git diff --quiet; then
        echo "⚠️  工作区有未提交的更改，请先处理"
        git status --short
        exit 1
    fi
    echo "🧪 运行代码质量检查..."
    echo "   1. 运行所有测试"
    if ! cargo test --quiet; then
        echo "❌ 测试失败，请修复后重试"
        exit 1
    fi
    echo "   2. 检查代码格式化"
    if ! cargo fmt -- --check; then
        echo "❌ 代码格式化不符合规范，请运行 cargo fmt 修复"
        exit 1
    fi
    echo "   3. 运行 Clippy 检查"
    if ! cargo clippy -- -D warnings; then
        echo "❌ Clippy 检查失败，请修复后重试"
        exit 1
    fi
    echo "✅ 代码质量检查通过"
    current=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "📌 当前版本: $current"
    echo "🔖 升级级别: {{level}}"
    
    # 升级版本
    cargo set-version --bump {{level}}
    new=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "✅ Cargo.toml 版本已更新: $current -> $new"
    if sed -i '' "s/version-$current/version-$new/g" README.md; then
        echo "✅ README.md 版本已更新: $current -> $new"
    fi
    echo ""
    echo "📋 将要执行的操作:"
    echo "   1. git add ."
    echo "   2. git commit -m \"chore: bump version to $new\""
    echo "   3. git tag v$new"
    echo "   4. git push origin main --tags"
    echo ""
    read -p "确认发布 v$new 到 npm? (y/N) " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ 已取消，回滚版本..."
        git checkout Cargo.toml
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

# 升级版本并发布
release level:
    @#!/bin/bash
    current=$(grep "^version" Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "📌 当前版本: $current"
    echo "🔖 升级级别: {{level}}"
    cargo set-version --bump {{level}}
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