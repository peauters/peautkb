use keyberon::hid::{HidDevice, Protocol, ReportType, Subclass};

#[rustfmt::skip]
const REPORT_DESCRIPTOR : &[u8] = &[
    0x05, 0x0C,        // Usage Page (Consumer)
    0x09, 0x01,        // Usage (Consumer Control)
    0xA1, 0x01,        // Collection (Application)
    0x19, 0xB2,        //   Usage Minimum (Record)
    0x2A, 0xCD, 0x00,  //   Usage Maximum (Play/Pause)
    0x15, 0x00,        //   Logical Minimum (0)
    0x26, 0xCD, 0x00,  //   Logical Maximum (205)
    0x95, 0x01,        //   Report Count (1)
    0x75, 0x08,        //   Report Size (8)
    0x81, 0x00,        //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
    0xC0,              // End Collection
];

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MediaKey {
    Record = 0x0B2,
    FastForward = 0x0B3,
    Rewind = 0x0B4,
    NextTrack = 0x0B5,
    PrevTrack = 0x0B6,
    Stop = 0x0B7,
    Eject = 0x0B8,
    RandomPlay = 0x0B9,
    StopEject = 0x0CC,
    PlayPause = 0x0CD,
}

#[derive(Default)]
pub struct MediaKeys {
    report: MediaKeyHidReport,
}

impl MediaKeys {
    pub fn set_report(&mut self, report: MediaKeyHidReport) -> bool {
        if report == self.report {
            false
        } else {
            self.report = report;
            true
        }
    }
}

impl HidDevice for MediaKeys {
    fn subclass(&self) -> Subclass {
        Subclass::None
    }

    fn protocol(&self) -> Protocol {
        Protocol::Keyboard
    }

    fn report_descriptor(&self) -> &[u8] {
        REPORT_DESCRIPTOR
    }

    fn get_report(&mut self, report_type: ReportType, _report_id: u8) -> Result<&[u8], ()> {
        match report_type {
            ReportType::Input => Ok(self.report.as_bytes()),
            _ => Err(()),
        }
    }

    fn set_report(
        &mut self,
        _report_type: ReportType,
        _report_id: u8,
        _data: &[u8],
    ) -> Result<(), ()> {
        Ok(())
    }
}

#[derive(Default, PartialEq, Copy, Clone)]
pub struct MediaKeyHidReport([u8; 2]);

impl MediaKeyHidReport {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<&MediaKey> for MediaKeyHidReport {
    fn from(key: &MediaKey) -> Self {
        let mut rep = MediaKeyHidReport::default();
        rep.0[0] = ((*key as u16) >> 8) as u8;
        rep.0[1] = *key as u8;
        rep
    }
}
