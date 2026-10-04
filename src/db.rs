use alloc::vec::Vec;
use uefi::CString16;
use uefi::runtime::{self, VariableVendor};

const MICROSOFT_VENDOR_GUID: [u8; 16] = [
    0xbd, 0x9a, 0xfa, 0x77, // Data1 (LE)
    0x59, 0x03, // Data2 (LE)
    0x32, 0x4d, // Data3 (LE)
    0xbd, 0x60, 0x28, 0xf4, 0xe7, 0x8f, 0x78, 0x4b,
];
/// 读取 UEFI `db` 变量的原始数据。
///
/// 返回的数据是标准的 `EFI_SIGNATURE_LIST` 序列，
/// 可以直接作为 MOKvar 表中 `"MokListRT"` 条目的内容。
pub fn read_db_variable() -> uefi::Result<Vec<u8>> {
    let name = CString16::try_from("db")
        .map_err(|_| uefi::Error::from(uefi::Status::INVALID_PARAMETER))?;

    // get_variable_boxed 直接返回 Box<[u8]>，无需预先查询大小。
    // 变量不存在时返回 NOT_FOUND。
    let (data, _attrs) =
        runtime::get_variable_boxed(&name, &VariableVendor::IMAGE_SECURITY_DATABASE)?;

    Ok(data.into_vec())
}

pub fn filter_microsoft_certs(db_data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = 0usize;

    while pos + 28 <= db_data.len() {
        // EFI_SIGNATURE_LIST 头: signature_type(16) + list_size(4) + header_size(4) + signature_size(4)
        let list_size = u32::from_le_bytes([
            db_data[pos + 16],
            db_data[pos + 17],
            db_data[pos + 18],
            db_data[pos + 19],
        ]) as usize;
        let sig_size = u32::from_le_bytes([
            db_data[pos + 24],
            db_data[pos + 25],
            db_data[pos + 26],
            db_data[pos + 27],
        ]) as usize;

        if list_size == 0 || pos + list_size > db_data.len() || sig_size < 16 {
            break;
        }

        let list_end = pos + list_size;
        let header_end = pos + 28; // SignatureHeaderSize 通常为 0

        // 重建这个 list 的头（签名类型和大小保持原样）
        let mut new_list = Vec::new();
        new_list.extend_from_slice(&db_data[pos..header_end]);

        let mut sig_pos = header_end;
        while sig_pos + sig_size <= list_end {
            let owner = &db_data[sig_pos..sig_pos + 16];
            if owner != MICROSOFT_VENDOR_GUID {
                new_list.extend_from_slice(&db_data[sig_pos..sig_pos + sig_size]);
            }
            sig_pos += sig_size;
        }

        // 只在 list 里还有非 Microsoft 签名时才保留
        if new_list.len() > 28 {
            // 更新 list_size
            let new_size = new_list.len() as u32;
            new_list[16..20].copy_from_slice(&new_size.to_le_bytes());
            out.extend_from_slice(&new_list);
        }

        pos = list_end;
    }

    out
}
