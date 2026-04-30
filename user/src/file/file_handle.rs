// 导入必要的模块
use std::fs::{File, OpenOptions};  // 文件操作相关模块
use std::io::{self, Read, Seek, Write};  // IO 操作相关模块
use std::path::PathBuf;           // 路径操作相关模块
use std::marker::PhantomData;      // 用于在结构体中保存类型信息而不占用内存

// 标记类型（Marker Types）：用于在编译时标记文件的访问模式
// 这些类型不包含任何字段，仅用于类型系统中的类型标记
pub struct Readable;
pub struct Writable;
pub struct ReadWritable;

// 泛型文件处理器结构体
// Mode 是一个类型参数，用于指定文件的访问模式
pub struct FileHander<Mode> {
    file: File,             // 底层的 File 对象
    file_path: PathBuf,     // 文件路径
    _mode: PhantomData<Mode>,  // 幻影数据，用于在结构体中保存类型信息而不占用内存
}

// 为 ReadWritable 模式的 FileHander 实现方法
impl FileHander<ReadWritable> {
    // 以读写模式打开文件
    pub fn open_read_write(file_path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)     // 允许读取
            .write(true)    // 允许写入
            .create(true)   // 如果文件不存在则创建
            .open(file_path)?;  // 打开文件，返回 Result
            
        Ok(Self {
            file,
            file_path: file_path.into(),  // 将 &str 转换为 PathBuf
            _mode: PhantomData,  // 初始化幻影数据
        })
    }
}

// 为 Readable 模式的 FileHander 实现方法
impl FileHander<Readable> {
    // 以只读模式打开文件
    pub fn open_read(file_path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)     // 允许读取
            .open(file_path)?;  // 打开文件，返回 Result
            
        Ok(Self {
            file,
            file_path: file_path.into(),
            _mode: PhantomData,
        })
    }

    pub fn seek_to_start(&mut self) -> io::Result<()> {
        self.file.seek(io::SeekFrom::Start(0))?;
        Ok(())
    }
}

// 为 Writable 模式的 FileHander 实现方法
impl FileHander<Writable> {
    // 以写入模式打开文件（会截断文件内容）
    pub fn open_write(file_path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)    // 允许写入
            .create(true)   // 如果文件不存在则创建
            .truncate(true) // 截断文件内容
            .open(file_path)?;
            
        Ok(Self {
            file,
            file_path: file_path.into(),
            _mode: PhantomData,
        })
    }
    
    // 以追加模式打开文件
    pub fn open_append(file_path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .append(true)   // 允许追加写入
            .create(true)   // 如果文件不存在则创建
            .open(file_path)?;
            
        Ok(Self {
            file,
            file_path: file_path.into(),
            _mode: PhantomData,
        })
    }
}

// 为所有模式的 FileHander 实现共同的方法
impl<Mode> FileHander<Mode> {
    // 获取文件路径
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
}

// 只有可写的模式才能写入（使用 trait bound 进行限制）
impl<Mode> FileHander<Mode> where Self: CanWrite {
    // 写入字符串切片
    pub fn write_str(&mut self, value: &str) -> io::Result<()> {
        self.file.write_all(value.as_bytes())?;  // 写入字节
        self.file.flush()?;  // 刷新缓冲区
        Ok(())
    }

    // 写入 String
    pub fn write_string(&mut self, value: String) -> io::Result<()> {
        self.write_str(value.as_str())  // 调用 write_str 方法
    }

    // 清除文件内容
    pub fn clear(&mut self) -> io::Result<()> {
        self.file.set_len(0)?;  // 设置文件长度为 0，清除内容
        Ok(())
    }
}

// 只有可读的模式才能读取（使用 trait bound 进行限制）
impl<Mode> FileHander<Mode> where Self: CanRead {
    // 读取文件内容到字符串
    pub fn read_to_string(&mut self) -> io::Result<String> {
        let mut content = String::new();  // 创建空字符串
        self.file.read_to_string(&mut content)?;  // 读取内容
        Ok(content)
    }
}

// 标记 trait（Marker Traits）：用于标记哪些模式可以读取或写入
pub trait CanRead {}
pub trait CanWrite {}

// 为 Readable 和 ReadWritable 模式实现 CanRead trait
impl CanRead for FileHander<Readable> {}
impl CanRead for FileHander<ReadWritable> {}

// 为 Writable 和 ReadWritable 模式实现 CanWrite trait
impl CanWrite for FileHander<Writable> {}
impl CanWrite for FileHander<ReadWritable> {}

// 模式转换：从 Readable 模式转换为 Writable 模式
impl FileHander<Readable> {
    pub fn into_writeable(self) -> io::Result<FileHander<Writable>> {
        let file = OpenOptions::new()
            .write(true)
            .open(&self.file_path)?;
            
        Ok(FileHander {
            file,
            file_path: self.file_path,  // 转移所有权
            _mode: PhantomData,
        })
    }
}