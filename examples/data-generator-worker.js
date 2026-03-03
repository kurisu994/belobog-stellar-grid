// 数据生成 Worker
// 在后台线程生成大量表格数据，避免阻塞主线程

self.onmessage = function(e) {
    const { rowCount } = e.data;

    const categories = ['电子产品', '家居用品', '图书', '服装', '食品', '运动器材'];
    const products = [
        '智能手机', '笔记本电脑', '平板电脑', '耳机', '充电器',
        '沙发', '床', '桌子', '椅子', '台灯',
        '小说', '技术书籍', '漫画', '杂志', '词典',
        'T恤', '裤子', '外套', '鞋子', '帽子',
        '零食', '饮料', '水果', '蔬菜', '肉类',
        '跑步机', '哑铃', '瑜伽垫', '运动鞋', '运动服'
    ];

    const rows = [];
    const batchSize = 1000; // 每批处理 1000 行
    let processedCount = 0;

    for (let i = 1; i <= rowCount; i++) {
        const category = categories[Math.floor(Math.random() * categories.length)];
        const product = products[Math.floor(Math.random() * products.length)];
        const price = (Math.random() * 1000 + 50).toFixed(2);
        const stock = Math.floor(Math.random() * 1000);
        const sales = Math.floor(Math.random() * 500);

        rows.push({
            id: `P${i.toString().padStart(4, '0')}`,
            product,
            category,
            price: `¥${price}`,
            stock,
            sales
        });

        processedCount++;

        // 每处理一批数据，发送进度更新
        if (processedCount % batchSize === 0) {
            const progress = (processedCount / rowCount) * 100;
            self.postMessage({
                type: 'progress',
                progress: progress.toFixed(1),
                processedCount,
                totalCount: rowCount
            });
        }
    }

    // 发送完成消息
    self.postMessage({
        type: 'complete',
        rows,
        totalCount: rowCount
    });
};

