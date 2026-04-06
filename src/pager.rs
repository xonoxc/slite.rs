use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    usize,
};

use crate::{
    data::table::{PAGE_SIZE, TABLE_MAX_PAGES},
    errors::PagerError,
};

#[derive(Debug)]
pub struct Pager {
    pub file: File,
    pub file_length: u64,
    pub num_pages: usize,
    pub pages: Vec<Option<[u8; PAGE_SIZE]>>,
}

impl Pager {
    pub fn new(db_file_path: &str) -> Result<Self, PagerError> {
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(db_file_path)
            .expect("failed to open database file. Please make sure it exists..");

        let file_length = file.metadata().unwrap().len();
        let num_pages = (file_length as usize) / PAGE_SIZE;

        if file_length % (PAGE_SIZE as u64) != 0 {
            return Err(PagerError::InitError {
                cause: "DB file corrupt. Invalid db file.".to_string(),
            });
        }

        Ok(Self {
            file,
            file_length,
            num_pages,
            pages: vec![None],
        })
    }

    pub fn get_two_pages<const N: usize>(
        &mut self,
        mut page_nums: [usize; N],
    ) -> Result<[&mut [u8; PAGE_SIZE]; N], PagerError> {
        page_nums.sort_unstable();

        for i in 1..N {
            if page_nums[i] == page_nums[i - 1] {
                panic!("duplicate page access");
            }
        }

        for &page_num in &page_nums {
            if page_num >= TABLE_MAX_PAGES {
                return Err(PagerError::OutBoundSeek {
                    page: page_num,
                    max_allowed_pages: TABLE_MAX_PAGES,
                });
            }

            if page_num >= self.pages.len() {
                self.pages.resize(page_num + 1, None);
            }

            if self.pages[page_num].is_none() {
                let page_size = PAGE_SIZE as u64;

                let mut number_of_pages = self.file_length / page_size;
                if self.file_length % page_size > 0 {
                    number_of_pages += 1;
                }

                if (page_num as u64) < number_of_pages {
                    self.seek_file_to_offset(self.get_offset(page_num));
                    self.pages[page_num] = Some(self.read_into_buffer());
                } else {
                    self.pages[page_num] = Some([0; PAGE_SIZE]);
                }
            }
        }

        let mut result: [*mut [u8; PAGE_SIZE]; N] = std::array::from_fn(|_| std::ptr::null_mut());
        let mut slice = &mut self.pages[..];

        for (i, &page_num) in page_nums.iter().enumerate() {
            let (left, right) = slice.split_at_mut(page_num);
            let page = right[0].as_mut().unwrap();

            result[i] = page as *mut _;
            slice = left;
        }

        Ok(result.map(|ptr| unsafe { &mut *ptr }))
    }

    pub fn get_page(&mut self, page_num: usize) -> Result<&mut [u8; PAGE_SIZE], PagerError> {
        if page_num >= TABLE_MAX_PAGES {
            return Err(PagerError::OutBoundSeek {
                page: page_num,
                max_allowed_pages: TABLE_MAX_PAGES,
            });
        }

        if page_num >= self.num_pages {
            self.pages.resize(page_num + 1, None);
        }

        if self.pages[page_num].is_none() {
            let i64_page_size = PAGE_SIZE as u64;

            let mut number_of_pages = self.file_length / i64_page_size;

            if (self.file_length % i64_page_size) > 0 {
                number_of_pages += 1
            }

            if (page_num as u64) < number_of_pages {
                self.seek_file_to_offset(self.get_offset(page_num));

                self.pages[page_num] = Some(self.read_into_buffer());
            } else {
                self.pages[page_num] = Some([0; PAGE_SIZE]);
            }
        }

        Ok(self.pages[page_num].as_mut().unwrap())
    }

    pub fn allocate_page(&mut self, page_num: usize) {
        if page_num >= self.num_pages {
            self.pages.resize(page_num + 1, None);
        }

        self.pages[page_num] = Some([0; PAGE_SIZE])
    }

    pub fn flush(&mut self, page_num: usize) -> Result<(), PagerError> {
        self.seek_file_to_offset(self.get_offset(page_num));

        let page = match self.pages.get_mut(page_num) {
            Some(Some(page)) => page,
            _ => return Err(PagerError::FlushError { page_num }),
        };

        self.file
            .write_all(page)
            .map_err(|_| PagerError::FlushError { page_num })?;

        self.file
            .sync_all()
            .map_err(|_| PagerError::FlushError { page_num })?;

        Ok(())
    }

    fn get_offset(&self, page_num: usize) -> u64 {
        (page_num as u64) * (PAGE_SIZE as u64)
    }

    fn seek_file_to_offset(&mut self, offset: u64) {
        self.file
            .seek(std::io::SeekFrom::Start(offset))
            .expect("cannot seek page offset");
    }

    fn read_into_buffer(&mut self) -> [u8; PAGE_SIZE] {
        let mut page_buf = [0u8; PAGE_SIZE];

        self.file
            .read_exact(&mut page_buf)
            .expect("error reading page from file");

        page_buf
    }
}
