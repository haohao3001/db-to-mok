#![no_main]
#![no_std]

extern crate alloc;

mod db;
mod loader;
mod mokvar;

use uefi::prelude::*;

#[entry]
fn main() -> Status {
    // 0.36 版本不再需要传入 system_table
    uefi::helpers::init().unwrap();

    // -------- Step 1: 读取 UEFI db 变量 --------
    let db_data = match db::read_db_variable() {
        Ok(data) if !data.is_empty() => data,
        Ok(_) => {
            log::error!("db 变量为空");
            return Status::ABORTED;
        }
        Err(e) => {
            log::error!("读取 db 变量失败: {:?}", e);
            return Status::ABORTED;
        }
    };
    let filtered = db::filter_microsoft_certs(&db_data);
    log::info!("从 db 读取到 {} 字节", filtered.len());

    // -------- Step 2: 构建 MOKvar 表 --------
    let table = mokvar::build_mokvar_table(&[("MokListRT", &filtered), ("MokListTrustedRT", &[])]);
    log::info!("MOKvar 表大小 = {} 字节", table.len());

    // -------- Step 3: 安装 MOKvar 配置表 --------
    if let Err(e) = mokvar::install_mokvar_table(&table) {
        log::error!("安装 MOKvar 配置表失败: {:?}", e);
        return Status::ABORTED;
    }
    log::info!("MOKvar 配置表安装完成");

    // -------- Step 4: 加载并启动 UKI --------
    if let Err(e) = loader::load_and_start_uki() {
        log::error!("加载 arch-linux.efi 失败: {:?}", e);
        return Status::ABORTED;
    }

    Status::SUCCESS
}
