//! A folder's contents as Explorer lists them, for the dock's folder menu,
//! and the standard dialog to choose that folder.

use std::{cmp::Ordering, fs, io, os::windows::fs::MetadataExt};

use windows::{
    Win32::{
        Foundation::{E_FAIL, HWND},
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize,
        },
        UI::{
            Shell::{
                FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, FileOpenDialog, IFileOpenDialog, IShellItem,
                SHCreateItemFromParsingName, SHFILEINFOW, SHGFI_DISPLAYNAME, SHGFI_ICON,
                SHGFI_SMALLICON, SHGetFileInfoW, SIGDN_FILESYSPATH, StrCmpLogicalW,
            },
            WindowsAndMessaging::DestroyIcon,
        },
    },
    core::HSTRING,
};

use crate::icons::{IconImage, icon_image};

const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;

/// An item of a folder, as Explorer shows it.
#[derive(Clone, Debug)]
pub struct FolderItem {
    pub path: String,
    /// The name Explorer shows: without `.lnk`, and without the extension
    /// when Explorer hides extensions.
    pub name: String,
    pub is_folder: bool,
    /// Explorer's small icon, `None` if the shell has none.
    pub icon: Option<IconImage>,
}

/// The items of `folder`: folders first, then files, each in Explorer's
/// order ("2" before "10"). Hidden and system items are left out, as in
/// Explorer by default (`desktop.ini`).
///
/// Requires COM to be initialized on the calling thread.
pub fn list(folder: &str) -> io::Result<Vec<FolderItem>> {
    let mut items: Vec<FolderItem> = fs::read_dir(folder)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.metadata().is_ok_and(|meta| {
                meta.file_attributes() & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM) == 0
            })
        })
        .map(|entry| {
            let path = entry.path();
            // Through links too: a junction to a folder opens like one.
            let is_folder = fs::metadata(&path).is_ok_and(|meta| meta.is_dir());
            let path = path.to_string_lossy().into_owned();
            let (name, icon) = shell_info(&path);
            FolderItem {
                name: name.unwrap_or_else(|| entry.file_name().to_string_lossy().into_owned()),
                path,
                is_folder,
                icon,
            }
        })
        .collect();
    items.sort_by(explorer_order);
    Ok(items)
}

fn explorer_order(a: &FolderItem, b: &FolderItem) -> Ordering {
    b.is_folder.cmp(&a.is_folder).then_with(|| {
        // SAFETY: Both strings live for the call.
        unsafe { StrCmpLogicalW(&HSTRING::from(&a.name), &HSTRING::from(&b.name)) }.cmp(&0)
    })
}

/// The display name and small icon the shell gives `path`.
fn shell_info(path: &str) -> (Option<String>, Option<IconImage>) {
    let mut info = SHFILEINFOW::default();
    // SAFETY: `info` is a buffer of the declared size; the icon written to it
    // is ours and destroyed below.
    let found = unsafe {
        SHGetFileInfoW(
            &HSTRING::from(path),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&raw mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON | SHGFI_DISPLAYNAME,
        )
    };
    if found == 0 {
        return (None, None);
    }
    let icon = (!info.hIcon.is_invalid()).then(|| {
        let image = icon_image(info.hIcon);
        // SAFETY: SHGetFileInfo gave us this icon; it is not used afterward.
        unsafe {
            let _ = DestroyIcon(info.hIcon);
        }
        image
    });
    let length = info
        .szDisplayName
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(info.szDisplayName.len());
    let name = String::from_utf16_lossy(&info.szDisplayName[..length]);
    ((!name.is_empty()).then_some(name), icon.flatten())
}

/// Asks for a folder with the standard Windows dialog, opened at `initial`;
/// `None` if the user cancels. `owner` is the window the dialog belongs to.
///
/// Initializes COM on the calling thread for the duration of the call.
#[must_use]
pub fn pick(owner: isize, initial: Option<&str>) -> Option<String> {
    // SAFETY: Balanced below, once every COM object is released.
    let com = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    let picked = pick_in_apartment(owner, initial);
    if com.is_ok() {
        // SAFETY: Balances the successful initialization above.
        unsafe { CoUninitialize() };
    }
    picked.ok()
}

fn pick_in_apartment(owner: isize, initial: Option<&str>) -> windows::core::Result<String> {
    // SAFETY: COM calls with arguments that live for each call; the path the
    // dialog returns is freed once copied.
    unsafe {
        let dialog: IFileOpenDialog =
            CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
        dialog.SetOptions(dialog.GetOptions()? | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)?;
        if let Some(start) = initial.and_then(|path| {
            SHCreateItemFromParsingName::<_, _, IShellItem>(&HSTRING::from(path), None).ok()
        }) {
            let _ = dialog.SetFolder(&start);
        }
        // Cancelling is an error here, and ends the pick.
        dialog.Show(Some(HWND(owner as *mut _)))?;
        let raw = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
        let path = raw.to_string();
        CoTaskMemFree(Some(raw.0.cast()));
        path.map_err(|_| E_FAIL.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, is_folder: bool) -> FolderItem {
        FolderItem {
            path: name.into(),
            name: name.into(),
            is_folder,
            icon: None,
        }
    }

    #[test]
    fn explorer_order_should_put_folders_first_then_sort_numbers_by_value() {
        let mut items = [
            item("file10", false),
            item("Zeta", true),
            item("file2", false),
            item("alpha", true),
            item("Beta", false),
        ];
        items.sort_by(explorer_order);
        let names: Vec<&str> = items.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, ["alpha", "Zeta", "Beta", "file2", "file10"]);
    }

    #[test]
    fn list_should_leave_out_hidden_items_and_mark_folders() -> io::Result<()> {
        // SAFETY: The shell needs COM on this test thread.
        let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        let root = std::env::temp_dir().join(format!("muelle-folder-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("Sub"))?;
        fs::write(root.join("note.txt"), b"x")?;
        fs::write(root.join("hidden.txt"), b"x")?;
        let hidden = HSTRING::from(root.join("hidden.txt").to_string_lossy().as_ref());
        // SAFETY: The path lives for the call.
        unsafe {
            windows::Win32::Storage::FileSystem::SetFileAttributesW(
                &hidden,
                FILE_FLAGS_AND_ATTRIBUTES(FILE_ATTRIBUTE_HIDDEN),
            )?;
        }
        let items = list(&root.to_string_lossy())?;
        let _ = fs::remove_dir_all(&root);
        let found: Vec<(bool, bool)> = items
            .iter()
            .map(|item| (item.is_folder, item.path.ends_with("note.txt")))
            .collect();
        assert_eq!(found, [(true, false), (false, true)]);
        Ok(())
    }
}
