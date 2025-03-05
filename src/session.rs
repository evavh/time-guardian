use windows::Win32::System::RemoteDesktop::{
    self, WTSQuerySessionInformationA, WTS_CURRENT_SERVER_HANDLE,
    WTS_SESSION_INFOA,
};
use windows_core::PSTR;

#[derive(Debug)]
pub(crate) struct Session {
    pub(crate) id: u32,
    pub(crate) station_name: String,
    pub(crate) active: bool,
    pub(crate) username: Option<String>,
}

impl From<WTS_SESSION_INFOA> for Session {
    fn from(item: WTS_SESSION_INFOA) -> Self {
        let id = item.SessionId;
        let station_name = unsafe { item.pWinStationName.to_string().unwrap() };
        let active = item.State.0 == 0;

        Session {
            id,
            station_name,
            active,
            username: None,
        }
    }
}

impl Session {
    pub(crate) unsafe fn add_username(mut self) -> Self {
        self.username = Some(unsafe { username_from_session_id(self.id) });
        self
    }
}

/// Unsafe
pub(crate) fn get_sessions() -> Vec<Session> {
    let capacity = 256;
    let mut sessions: Vec<WTS_SESSION_INFOA> = Vec::with_capacity(capacity);
    let mut sessions: *mut _ = sessions.as_mut_ptr();
    let mut n_sessions = 0;
    let n_sessions: *mut u32 = &mut n_sessions;

    let sessions = unsafe {
        RemoteDesktop::WTSEnumerateSessionsA(
            WTS_CURRENT_SERVER_HANDLE,
            0, // Does nothing
            1, // Does nothing
            &mut sessions,
            n_sessions,
        )
        .unwrap();

        Vec::from_raw_parts(
            sessions,
            (*n_sessions).try_into().unwrap(),
            capacity,
        )
    };

    sessions
        .into_iter()
        .map(|s| unsafe { Session::from(s).add_username() })
        .collect()
}

pub(crate) unsafe fn username_from_session_id(id: u32) -> String {
    let mut buffer: Vec<u8> = Vec::with_capacity(256);
    let username: *mut PSTR = &mut PSTR::from_raw(buffer.as_mut_ptr());
    let mut username_len = 0;
    let username_len: *mut u32 = &mut username_len;

    let username = unsafe {
        WTSQuerySessionInformationA(
            WTS_CURRENT_SERVER_HANDLE,
            id,
            RemoteDesktop::WTSUserName,
            username,
            username_len,
        )
        .unwrap();
        username
    };

    (*username).to_string().unwrap()
}
