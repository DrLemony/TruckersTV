//! The game's executable as mapped in its process, for finding things in its code by their shape.

/// The game's executable, as mapped in its process.
pub(crate) struct Image {
    base: usize,
    /// The whole image's size as mapped (its `SizeOfImage`): nothing is read outside it.
    size: usize,
    text: (usize, usize),
    rdata: (usize, usize),
    /// Its unwind table (`.pdata`): where each of its functions starts and ends.
    pdata: Option<(usize, usize)>,
}

impl Image {
    /// The image mapped at `base`: its code, its constants and its unwind table.
    ///
    /// # Safety
    /// `base` is the start of a module mapped in this process (a PE image), for as long as the `Image` lives.
    pub(crate) unsafe fn of(base: usize) -> Option<Self> {
        // SAFETY: the PE headers lie at the start of the mapped image (the caller's contract), read as they lie.
        let u16_at = |at: usize| unsafe { (base as *const u8).add(at).cast::<u16>().read_unaligned() } as usize;
        // SAFETY: as above.
        let u32_at = |at: usize| unsafe { (base as *const u8).add(at).cast::<u32>().read_unaligned() } as usize;
        let header = u32_at(0x3c);
        let (count, optional) = (u16_at(header + 6), u16_at(header + 20));
        let section = |name: &[u8]| {
            (0..count).find_map(|i| {
                let at = header + 24 + optional + i * 40;
                // SAFETY: a section header's name, within the headers the image maps.
                let bytes = unsafe { std::slice::from_raw_parts((base + at) as *const u8, 8) };
                bytes.starts_with(name).then(|| (base + u32_at(at + 12), u32_at(at + 8)))
            })
        };
        Some(Image {
            base,
            // The optional header's SizeOfImage.
            size: u32_at(header + 24 + 56),
            text: section(b".text\0")?,
            rdata: section(b".rdata\0")?,
            pdata: section(b".pdata\0"),
        })
    }

    /// `len` bytes of the image at the address `at`, if they lie within it.
    pub(crate) fn code_at(&self, at: usize, len: usize) -> Option<&[u8]> {
        self.bytes(at.checked_sub(self.base)?, len)
    }

    /// `len` bytes of the image at `rva`, if they lie within it.
    fn bytes(&self, rva: usize, len: usize) -> Option<&[u8]> {
        // SAFETY: within the mapped image (checked against its size), mapped for the process's life.
        (rva.checked_add(len)? <= self.size)
            .then(|| unsafe { std::slice::from_raw_parts((self.base + rva) as *const u8, len) })
    }

    /// Where the function holding the code at `at` starts, from the unwind table: a function in several parts
    /// (its later parts' unwind info chained to the one before) starts where its first part does.
    pub(crate) fn function_start(&self, at: usize) -> Option<usize> {
        let (table, len) = self.pdata?;
        let rva = at.checked_sub(self.base)?;
        let entry = |i: usize| {
            // SAFETY: an entry of the unwind table, within the section (`i` below its count).
            let read = |field: usize| unsafe { ((table + i * 12 + field * 4) as *const u32).read_unaligned() } as usize;
            (read(0), read(1), read(2))
        };
        // The entries are sorted by where their code starts: the last one starting at or before `rva`.
        let (mut low, mut high) = (0, len / 12);
        while low < high {
            let middle = (low + high) / 2;
            if entry(middle).0 <= rva { low = middle + 1 } else { high = middle }
        }
        let (mut start, end, mut unwind) = entry(low.checked_sub(1)?);
        if rva >= end {
            return None;
        }
        for _ in 0..16 {
            // UNWIND_INFO: its flags in the first byte's top five bits, its count of unwind codes in the third;
            // chained (UNW_FLAG_CHAININFO), the part before's entry follows the codes (kept to an even count).
            let info = self.bytes(unwind, 4)?;
            if (info[0] >> 3) & 0x4 == 0 {
                return Some(self.base + start);
            }
            let parent = self.bytes(unwind + 4 + (usize::from(info[2]).next_multiple_of(2)) * 2, 12)?;
            let field = |i: usize| u32::from_le_bytes(parent[i * 4..i * 4 + 4].try_into().unwrap()) as usize;
            (start, unwind) = (field(0), field(2));
        }
        None
    }

    fn code(&self) -> &[u8] {
        // SAFETY: the image's code section, mapped for the process's life (`Image::of`).
        unsafe { std::slice::from_raw_parts(self.text.0 as *const u8, self.text.1) }
    }

    fn constants(&self) -> &[u8] {
        // SAFETY: the image's constants section, mapped for the process's life (`Image::of`).
        unsafe { std::slice::from_raw_parts(self.rdata.0 as *const u8, self.rdata.1) }
    }

    /// Where `pattern` starts in the code, if exactly once (`None` in it: any byte).
    pub(crate) fn unique_masked(&self, pattern: &[Option<u8>]) -> Option<usize> {
        let fits = |window: &[u8]| window.iter().zip(pattern).all(|(byte, wanted)| wanted.is_none_or(|w| *byte == w));
        let mut found = self.code().windows(pattern.len()).enumerate().filter(|(_, w)| fits(w));
        let (at, _) = found.next()?;
        found.next().is_none().then_some(self.text.0 + at)
    }

    /// Where a RIP-relative lea at `at` (an address in the code) points.
    fn lea_target(&self, at: usize) -> Option<usize> {
        let offset = at.checked_sub(self.text.0)?;
        let b = self.code().get(offset..offset + 7)?;
        let rip_relative = (b[0] == 0x48 || b[0] == 0x4c) && b[1] == 0x8d && b[2] & 0xc7 == 0x05;
        rip_relative.then(|| (at + 7).wrapping_add_signed(i32::from_le_bytes(b[3..7].try_into().unwrap()) as isize))
    }

    pub(crate) fn constant(&self, text: &[u8]) -> Option<usize> {
        self.constants().windows(text.len()).position(|w| w == text).map(|at| self.rdata.0 + at)
    }

    /// The code loading the address `target` (a RIP-relative lea), if exactly one place does.
    pub(crate) fn unique_load(&self, target: usize) -> Option<usize> {
        let mut found = (0..self.code().len().saturating_sub(7))
            .map(|i| self.text.0 + i)
            .filter(|&at| self.lea_target(at) == Some(target));
        let at = found.next()?;
        found.next().is_none().then_some(at)
    }
}
