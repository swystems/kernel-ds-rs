use kernel::str::CString;
use kernel::fmt;

#[allow(dead_code)]
pub(crate) struct Message {
    content: CString,
}

#[allow(dead_code)]
impl Message {
    pub(crate) fn from_msgid_with_size(msgid: u64, size: usize) -> Message {
        let content = CString::try_from_fmt(fmt!("{0:>1$}", msgid, size-1)).unwrap();
        assert_eq!(content.as_bytes_with_nul().len(), size);
        Message{content}
    }

    pub(crate) fn _to_str(&self) -> &str {
        self.content.to_str().unwrap()
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes_with_nul()
    }
}