use alloc::vec::Vec;
use core::ffi::c_void;
use uefi::Guid;
use uefi::boot::{self, MemoryType};
use uefi::guid;

/// LINUX_EFI_MOK_VARIABLE_TABLE_GUID
/// 内核 arch/x86/include/.../efi.h 或 include/linux/efi.h 中的定义
pub const LINUX_EFI_MOK_VARIABLE_TABLE_GUID: Guid = guid!("c451ed2b-9694-45d3-baba-ed9f8988a389");

/// struct efi_mokvar_table_entry 中 name 数组的长度（内核定为 256）
const ENTRY_NAME_SIZE: usize = 256;

/// 与内核 struct efi_mokvar_table_entry 布局一致
/// （内核是 packed，这里没有 padding 因为字段天然对齐）
#[repr(C, packed)]
pub struct EfiMokvarTableEntry {
    pub name: [u8; ENTRY_NAME_SIZE],
    pub data_size: u64,
}

impl EfiMokvarTableEntry {
    /// 头大小（不含 data）
    pub const HEADER_SIZE: usize = core::mem::size_of::<Self>();
}

/// 构建完整的 MOKvar 配置表缓冲区
///
/// entries: (名称, 数据) 列表
/// 末尾会自动追加哨兵条目（name 全 0，data_size = 0）
pub fn build_mokvar_table(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    for (name, data) in entries {
        // name: 固定 256 字节，NUL 填充
        let mut name_buf = [0u8; ENTRY_NAME_SIZE];
        let bytes = name.as_bytes();
        let n = bytes.len().min(ENTRY_NAME_SIZE - 1);
        name_buf[..n].copy_from_slice(&bytes[..n]);
        buf.extend_from_slice(&name_buf);

        // data_size: u64 小端
        buf.extend_from_slice(&(data.len() as u64).to_le_bytes());

        // data
        buf.extend_from_slice(data);
    }

    // 哨兵条目
    buf.extend_from_slice(&[0u8; ENTRY_NAME_SIZE]);
    buf.extend_from_slice(&0u64.to_le_bytes());

    buf
}

/// 安装 MOKvar 配置表
///
/// 关键点：
/// 1. 必须分配 EFI_BOOT_SERVICES_DATA 类型的内存，内核才会 reserve 它
/// 2. 表必须落在单个 EFI memory descriptor 内（allocate_pool 一般满足）
/// 3. install_configuration_table 传入的是物理地址
pub fn install_mokvar_table(table: &[u8]) -> uefi::Result<()> {
    let pool = boot::allocate_pool(MemoryType::BOOT_SERVICES_DATA, table.len())?;

    unsafe {
        core::ptr::copy_nonoverlapping(table.as_ptr(), pool.as_ptr(), table.len());

        boot::install_configuration_table(
            &LINUX_EFI_MOK_VARIABLE_TABLE_GUID,
            pool.as_ptr() as *mut c_void,
        )?;
    }

    Ok(())
}
