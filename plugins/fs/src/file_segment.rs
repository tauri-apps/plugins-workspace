use std::fs::File;

pub struct FileSegment{
    pub file: File, 
    pub offset: u64, 
    pub size: u64
}

pub enum FileOrSegment {
    File(File),
    Segment(FileSegment),
}