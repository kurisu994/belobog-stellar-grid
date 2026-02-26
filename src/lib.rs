mod batch_export;
mod batch_export_xlsx;
mod core;
mod resource;
mod utils;
mod validation;

// 重新导出所有公共 API
pub use resource::UrlGuard;
pub use validation::{ensure_extension, validate_filename};

// 导出新的统一接口
pub use core::{ExportFormat, export_data, export_table, export_tables_xlsx, generate_data_bytes};

// 导出分批异步导出
pub use batch_export::export_table_to_csv_batch;
pub use batch_export_xlsx::{export_table_to_xlsx_batch, export_tables_to_xlsx_batch};

// 导出 utils 模块的公共函数
pub use utils::{escape_csv_injection, set_panic_hook};
