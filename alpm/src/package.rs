use crate::utils::*;
use crate::{
    Alpm, AlpmList, AlpmListMut, Backup, ChangeLog, Db, Dep, FileList, PackageFrom, PackageReason,
    PackageValidation, Result, Signature, Ver,
};

#[cfg(feature = "mtree")]
use crate::MTree;

use std::mem::transmute;
use std::ops::Deref;
use std::{fmt, ptr};

use alpm_sys::*;

pub trait AsPkg {
    fn as_pkg(&self) -> Pkg;
}

impl<'a> AsPkg for Package<'a> {
    fn as_pkg(&self) -> Pkg {
        self.pkg
    }
}

impl<'a> AsPkg for Pkg<'a> {
    fn as_pkg(&self) -> Pkg {
        *self
    }
}

#[derive(Copy, Clone)]
pub struct Package<'a> {
    pub(crate) pkg: Pkg<'a>,
}

#[derive(Copy, Clone)]
pub struct Pkg<'a> {
    pub(crate) handle: &'a Alpm,
    pub(crate) pkg: *mut alpm_pkg_t,
}

impl<'a> fmt::Debug for Pkg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pkg")
            .field("name", &self.name())
            .field("version", &self.version())
            .finish()
    }
}

impl<'a> fmt::Debug for Package<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Package")
            .field("name", &self.name())
            .field("version", &self.version())
            .finish()
    }
}

impl<'a> Deref for Package<'a> {
    type Target = Pkg<'a>;
    fn deref(&self) -> &Self::Target {
        &self.pkg
    }
}

impl<'a> Package<'a> {
    pub(crate) unsafe fn new(handle: &Alpm, pkg: *mut alpm_pkg_t) -> Package {
        Package {
            pkg: Pkg { handle, pkg },
        }
    }
}

impl<'a> Pkg<'a> {
    pub fn name(&self) -> &'a str {
        let name = unsafe { alpm_pkg_get_name(self.pkg) };
        unsafe { from_cstr(name) }
    }

    pub fn check_md5sum(&self) -> Result<()> {
        self.handle
            .check_ret(unsafe { alpm_pkg_checkmd5sum(self.pkg) })
    }

    pub fn should_ignore(&self) -> bool {
        let ret = unsafe { alpm_pkg_should_ignore(self.handle.handle, self.pkg) };
        ret != 0
    }

    pub fn filename(&self) -> &'a str {
        let name = unsafe { alpm_pkg_get_filename(self.pkg) };
        unsafe { from_cstr_optional2(name) }
    }

    pub fn base(&self) -> Option<&'a str> {
        let base = unsafe { alpm_pkg_get_base(self.pkg) };
        unsafe { from_cstr_optional(base) }
    }

    pub fn version(&self) -> &'a Ver {
        let version = unsafe { alpm_pkg_get_version(self.pkg) };
        unsafe { Ver::from_ptr(version) }
    }

    pub fn origin(&self) -> PackageFrom {
        let origin = unsafe { alpm_pkg_get_origin(self.pkg) };
        unsafe { transmute::<_alpm_pkgfrom_t, PackageFrom>(origin) }
    }

    pub fn desc(&self) -> Option<&'a str> {
        let desc = unsafe { alpm_pkg_get_desc(self.pkg) };
        unsafe { from_cstr_optional(desc) }
    }

    pub fn url(&self) -> Option<&'a str> {
        let url = unsafe { alpm_pkg_get_url(self.pkg) };
        unsafe { from_cstr_optional(url) }
    }

    pub fn build_date(&self) -> i64 {
        let date = unsafe { alpm_pkg_get_builddate(self.pkg) };
        date as i64
    }

    pub fn install_date(&self) -> Option<i64> {
        let date = unsafe { alpm_pkg_get_installdate(self.pkg) };
        if date == 0 {
            None
        } else {
            Some(date as i64)
        }
    }

    pub fn packager(&self) -> Option<&'a str> {
        let packager = unsafe { alpm_pkg_get_packager(self.pkg) };
        unsafe { from_cstr_optional(packager) }
    }

    pub fn md5sum(&self) -> Option<&'a str> {
        let md5sum = unsafe { alpm_pkg_get_md5sum(self.pkg) };
        unsafe { from_cstr_optional(md5sum) }
    }

    pub fn sha256sum(&self) -> Option<&'a str> {
        let sha256sum = unsafe { alpm_pkg_get_sha256sum(self.pkg) };
        unsafe { from_cstr_optional(sha256sum) }
    }

    pub fn arch(&self) -> Option<&'a str> {
        let arch = unsafe { alpm_pkg_get_arch(self.pkg) };
        unsafe { from_cstr_optional(arch) }
    }

    pub fn size(&self) -> i64 {
        let size = unsafe { alpm_pkg_get_size(self.pkg) };
        size as i64
    }

    pub fn isize(&self) -> i64 {
        let size = unsafe { alpm_pkg_get_isize(self.pkg) };
        size as i64
    }

    pub fn reason(&self) -> PackageReason {
        let reason = unsafe { alpm_pkg_get_reason(self.pkg) };
        unsafe { transmute::<_alpm_pkgreason_t, PackageReason>(reason) }
    }

    pub fn validation(&self) -> PackageValidation {
        let validation = unsafe { alpm_pkg_get_validation(self.pkg) };
        PackageValidation::from_bits(validation as u32).unwrap()
    }

    pub fn licenses(&self) -> AlpmList<'a, &'a str> {
        let list = unsafe { alpm_pkg_get_licenses(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn groups(&self) -> AlpmList<'a, &'a str> {
        let list = unsafe { alpm_pkg_get_groups(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn depends(&self) -> AlpmList<'a, Dep<'a>> {
        let list = unsafe { alpm_pkg_get_depends(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn optdepends(&self) -> AlpmList<'a, Dep<'a>> {
        let list = unsafe { alpm_pkg_get_optdepends(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn checkdepends(&self) -> AlpmList<'a, Dep<'a>> {
        let list = unsafe { alpm_pkg_get_checkdepends(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn makedepends(&self) -> AlpmList<'a, Dep<'a>> {
        let list = unsafe { alpm_pkg_get_makedepends(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn conflicts(&self) -> AlpmList<'a, Dep<'a>> {
        let list = unsafe { alpm_pkg_get_conflicts(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn provides(&self) -> AlpmList<'a, Dep<'a>> {
        let list = unsafe { alpm_pkg_get_provides(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn replaces(&self) -> AlpmList<'a, Dep<'a>> {
        let list = unsafe { alpm_pkg_get_replaces(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn files(&self) -> FileList {
        let files = unsafe { *alpm_pkg_get_files(self.pkg) };
        FileList { inner: files }
    }

    pub fn backup(&self) -> AlpmList<'a, Backup> {
        let list = unsafe { alpm_pkg_get_backup(self.pkg) };
        AlpmList::from_parts(self.handle, list)
    }

    pub fn db(&self) -> Option<Db<'a>> {
        let db = unsafe { alpm_pkg_get_db(self.pkg) };
        self.handle.check_null(db).ok()?;
        Some(Db {
            handle: self.handle,
            db,
        })
    }

    pub fn changelog(&self) -> Result<ChangeLog> {
        let changelog = unsafe { alpm_pkg_changelog_open(self.pkg) };
        self.handle.check_null(changelog)?;

        let changelog = ChangeLog {
            pkg: self,
            stream: changelog,
        };

        Ok(changelog)
    }

    #[cfg(feature = "mtree")]
    pub fn mtree(&self) -> Result<MTree> {
        let archive = unsafe { alpm_pkg_mtree_open(self.pkg) };
        self.handle.check_null(archive)?;

        let archive = MTree { pkg: self, archive };

        Ok(archive)
    }

    pub fn required_by(&self) -> AlpmListMut<'a, String> {
        let list = unsafe { alpm_pkg_compute_requiredby(self.pkg) };
        AlpmListMut::from_parts(self.handle, list)
    }

    pub fn optional_for(&self) -> AlpmListMut<'a, String> {
        let list = unsafe { alpm_pkg_compute_optionalfor(self.pkg) };
        AlpmListMut::from_parts(self.handle, list)
    }

    pub fn base64_sig(&self) -> Option<&'a str> {
        let base64_sig = unsafe { alpm_pkg_get_base64_sig(self.pkg) };
        unsafe { from_cstr_optional(base64_sig) }
    }

    pub fn has_scriptlet(&self) -> bool {
        unsafe { alpm_pkg_has_scriptlet(self.pkg) != 0 }
    }

    pub fn sig(&self) -> Result<Signature> {
        let mut sig = ptr::null_mut();
        let mut len = 0;
        let ret = unsafe { alpm_pkg_get_sig(self.pkg, &mut sig, &mut len) };
        self.handle.check_ret(ret)?;
        let sig = Signature { sig, len };
        Ok(sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SigLevel;
    use std::io::Read;

    #[test]
    fn test_depends() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let depends = pkg
            .depends()
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            &depends,
            &["coreutils", "linux-firmware", "kmod", "mkinitcpio"]
        )
    }

    #[test]
    fn test_files() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.localdb();
        let pkg = db.pkg("filesystem").unwrap();
        let files = pkg.files();

        for file in files.files() {
            println!("{}", file.name());
        }

        assert!(files.contains("etc/").unwrap().is_some());
        assert_eq!(pkg.filename(), "");
    }

    #[test]
    fn test_groups() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("core", SigLevel::NONE).unwrap();
        let pkg = db.pkg("linux").unwrap();
        let groups = pkg.groups();

        assert_eq!(&groups.iter().collect::<Vec<_>>(), &["base"],)
    }

    #[test]
    fn test_backup() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.localdb();
        let pkg = db.pkg("pacman").unwrap();
        let backup = pkg.backup();
        assert_eq!(backup.first().unwrap().name(), "etc/pacman.conf");
    }

    #[test]
    fn test_rquired_by() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.register_syncdb("extra", SigLevel::NONE).unwrap();
        let pkg = db.pkg("ostree").unwrap();
        let optional = pkg
            .required_by()
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>();
        assert_eq!(&optional, &["flatpak"]);
    }

    #[test]
    fn test_changelog() {
        let handle = Alpm::new("/", "tests/db").unwrap();
        let db = handle.localdb();
        let pkg = db.pkg("vifm").unwrap();
        let mut s = String::new();
        let mut changelog = pkg.changelog().unwrap();
        changelog.read_to_string(&mut s).unwrap();
        assert!(s.contains("2010-02-15 Jaroslav Lichtblau <svetlemodry@archlinux.org>"));
    }
}
