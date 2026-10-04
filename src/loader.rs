use alloc::vec::Vec;
use uefi::boot::{self, LoadImageSource};
use uefi::cstr16;
use uefi::fs::FileSystem;

/// 从 ESP 加载并启动 UKI
pub fn load_and_start_uki() -> uefi::Result<()> {
    // 1. 打开本 UEFI 应用所在的文件系统
    let fs_handle = boot::get_image_file_system(boot::image_handle())?;
    let mut fs = FileSystem::new(fs_handle);

    // 2. 读取整个文件
    //    FileSystem::read 返回 Result<Vec<u8>, uefi::fs::Error>，
    //    而本函数返回 uefi::Result<()>（即 Result<(), uefi::Error>）。
    //    两种 Error 之间没有 From 转换，需要手动映射。
    let path = cstr16!("\\EFI\\Linux\\arch-linux.efi");
    let image_data: Vec<u8> = fs.read(path).map_err(|e| {
        log::error!("读取 arch-linux.efi 失败: {:?}", e);
        uefi::Error::from(uefi::Status::NOT_FOUND)
    })?;
    log::info!("arch-linux.efi 大小 = {} 字节", image_data.len());

    // 3. LoadImage（从内存缓冲区加载）
    let image_handle = boot::load_image(
        boot::image_handle(),
        LoadImageSource::FromBuffer {
            buffer: &image_data,
            file_path: None,
        },
    )?;

    log::info!("LoadImage 成功，开始 StartImage");

    // 4. StartImage —— 成功后不会返回（内核接管）
    boot::start_image(image_handle)?;

    Ok(())
}
