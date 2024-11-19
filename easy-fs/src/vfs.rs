use super::{
    block_cache_sync_all, get_block_cache, BlockDevice, DirEntry, DiskInode, DiskInodeType,
    EasyFileSystem, DIRENT_SZ,
};
use crate::BLOCK_SZ;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};
/// Virtual filesystem layer over easy-fs
pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Inode {
    /// Create a vfs inode
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EasyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }
    /// Call a function over a disk inode to read it
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }
    /// Call a function over a disk inode to modify it
    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }
    /// Find inode under a disk inode by name
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_id() as u32);
            }
        }
        None
    }
    /// Find inode under current inode by name
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }
    /// Increase the size of a disk inode
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    /// Create inode under current inode by name
    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(DiskInodeType::File);
            });
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release efs lock automatically by compiler
    }
    /// List inodes under current inode
    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                v.push(String::from(dirent.name()));
            }
            v
        })
    }
    /// Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }
    /// Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }
    /// Clear the data in current inode
    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
    /// inode 文件所在 inode 编号
    pub fn fstat_get_id(&self)->u64{
        let block_begin = self.fs.lock().inode_area_start_block as usize;
        let inode_size = core::mem::size_of::<DiskInode>();
        let inodes_per_block = BLOCK_SZ / inode_size;
        let inode = (self.block_id - block_begin) * inodes_per_block + self.block_offset / inode_size;
        inode as u64
    }
    /// 文件类型
    pub fn fstat_mode(&self)->u32{
        // directory
        let mut _fs = self.fs.lock();
        if self.read_disk_inode(|node| node.is_dir()){
            return 2;
        }
        // ordinary regular file
        else if self.read_disk_inode(|node| node.is_file()){
            return 1;
        }
        // null
        else {
            return 0;
        }
    }
    /// 硬链接数量，初始为1
    pub fn fstat_nlink(&self)->u32{
        let mut _fs = self.fs.lock();
        self.read_disk_inode(|node| node.reference_count)
    }
    ///
    pub fn add_link(&self,_old_name: &str, _new_name: &str) -> isize {
        let mut fs = self.fs.lock();
        let old_inode_id = self.read_disk_inode(|disk_inode|
            self.find_inode_id(_old_name, disk_inode)
        );
         // old_name should point to a valid file
        if old_inode_id.is_none() {
            return -1;
        }
        let old_inode_id = old_inode_id.unwrap();
        // new_name should not point to an existing file
        if self.read_disk_inode(|disk_inode| self.find_inode_id(_new_name, disk_inode)).is_some() {
            return -1;
        }
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(_new_name, old_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(old_inode_id);
        //提供对 inode 的线程安全、引用计数的共享访问。
        let node =Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        ));
        node.modify_disk_inode(|disk_inode| {
            // 增加引用计数
            disk_inode.increase_reference_count();
        });
        block_cache_sync_all();
        0

        // release efs lock automatically by compiler
    }
    ///
    pub fn remove_link(&self, name: &str) -> isize {
        let mut fs = self.fs.lock();
        // 查找目录项
        let inode = self.modify_disk_inode(|disk_inode| {
            if !disk_inode.is_dir() {
                // 非目录文件，直接返回错误
                return None;
            }
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            let mut target_inode = None;
            let mut dirent = DirEntry::empty();
    
            for i in 0..file_count {
                // 读取目录项
                assert_eq!(
                    disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device),
                    DIRENT_SZ,
                );
                if dirent.name() == name {
                    // 找到目标目录项，保存其 inode 编号
                    target_inode = Some(dirent.inode_id());
    
                    if i == file_count - 1 {
                        // 如果是最后一项，直接清空并减小目录大小
                        disk_inode.write_at(DIRENT_SZ * i, &[0u8; DIRENT_SZ], &self.block_device);
                    } else {
                        // 覆盖当前项为最后一项，并清空最后一项
                        let mut last_dirent = DirEntry::empty();
                        disk_inode.read_at(
                            DIRENT_SZ * (file_count - 1),
                            last_dirent.as_bytes_mut(),
                            &self.block_device,
                        );
                        disk_inode.write_at(DIRENT_SZ * i, last_dirent.as_bytes(), &self.block_device);
                        disk_inode.write_at(DIRENT_SZ * (file_count - 1), &[0u8; DIRENT_SZ], &self.block_device);
                    }
                    disk_inode.size -= DIRENT_SZ as u32;
                    break;
                }
            }
            target_inode
        });
    
        // 如果目录项不存在，返回错误
        if inode.is_none() {
            return -1;
        }
        let inode = inode.unwrap();
    
        // 获取要删除文件的 inode 并减少引用计数
        let (block_id, block_offset) = fs.get_disk_inode_pos(inode);
        let node = Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        ));
    
        node.modify_disk_inode(|disk_inode| {
            // 减少引用计数
            disk_inode.decrease_reference_count();
            if disk_inode.can_remove() {
                // 如果引用计数为 0，清理文件
                let size = disk_inode.size;
                let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
                assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
                for data_block in data_blocks_dealloc {
                    fs.dealloc_data(data_block);
                }
            }
        });
    
        // 同步缓存
        block_cache_sync_all();
        0
    }
    
}
