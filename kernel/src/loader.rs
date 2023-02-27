use super::cycle;
use super::sdc;
use super::uart;

fn read_unaligned<T>(addr: *const u32, byte_offset: usize) -> T {
    unsafe { ((addr as *mut u8).add(byte_offset) as *mut T).read_unaligned() }
}

fn read<T>(addr: *const u32, byte_offset: usize) -> T {
    unsafe { ((addr as *mut u8).add(byte_offset) as *mut T).read() }
}

fn array_to_u32<const N: usize>(b: &[u8; N]) -> u32 {
    b.iter().rev().fold(0u32, |acc, x| (acc << 8) + *x as u32)
}

pub fn load_kernel() -> u32 {
    let buf: *mut u32 = 0x2000_1000 as *mut u32;

    let s = sdc::read_sector(0, buf);
    if s != 0 {
        return 1;
    }
    let boot_sector: u32 = read_unaligned(buf, 0x1c6);

    let s = sdc::read_sector(boot_sector, buf);
    if s != 0 {
        return 2;
    }
    //let sector_per_cluster: u32 = read::<u8>(buf, 0x00d) as u32;
    let reserved_sector_count: u32 = read::<u16>(buf, 0x00e) as u32;
    let num_fat: u32 = read::<u8>(buf, 0x010) as u32;
    let root_dirent_count: u32 = read_unaligned::<u16>(buf, 0x011) as u32;
    //let root_dirent_count: u32 = read_unaligned::<u16>(buf, 0x011) as u32;
    let fat_sector_count: u32 = read::<u16>(buf, 0x016) as u32;

    let fat_start_sector: u32 = boot_sector + reserved_sector_count;
    let fats_sectors: u32 = num_fat * fat_sector_count;
    let root_dirent_start_sector: u32 = fat_start_sector + fats_sectors;
    uart::print(fat_start_sector);
    uart::puts(b" ");
    uart::print(root_dirent_start_sector);
    uart::puts(b" ");
    uart::print(root_dirent_count);
    uart::puts(b" #dirent\r\n");

    let mut j = 0;
    let mut kernel_start_cluster: u32 = 0;
    let mut kernel_file_size: u32 = 0;
    for _ in 0..root_dirent_count {
        if j == 0 {
            let s = sdc::read_sector(root_dirent_start_sector, buf);
            if s != 0 {
                uart::print(s);
                uart::puts(b" ");
                return 2;
            }
        }
        let name_head4: u32 = read::<u32>(buf, j);
        if (name_head4 & 0xff) == 0x00 {
            break; // last entry
        }
        if name_head4 == array_to_u32(b"KERN")
            && read::<u32>(buf, j + 4) == array_to_u32(b"EL  ")
            && (read::<u32>(buf, j + 8) & 0xFFFFFF) == array_to_u32(b"BIN")
        {
            kernel_start_cluster = read::<u16>(buf, j + 26) as u32;
            kernel_file_size = read::<u32>(buf, j + 28);
            break;
        }
        j = (j + 32) & 511
    }
    uart::print(kernel_start_cluster);
    uart::puts(b" ");
    uart::print(kernel_file_size);
    uart::puts(b" #kern\r\n");

    for i in 0..128 {
        uart::print(read::<u32>(buf, i * 4));
        uart::puts(b"\r\n");
    }
    0
}
